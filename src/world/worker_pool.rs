use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::chunk::{Block, CHUNK_VOLUME, ChunkMesh, Generator, Neighbours, mesh};

type ChunkCache = Arc<Mutex<HashMap<(i32, i32, i32), Arc<[Block; CHUNK_VOLUME]>>>>;

pub struct WorkItem {
    pub coords: (i32, i32, i32),
}

pub struct WorkResult {
    pub coords: (i32, i32, i32),
    pub mesh: ChunkMesh,
}

pub struct WorkerPool {
    senders: Vec<mpsc::Sender<WorkItem>>,
    handles: Vec<JoinHandle<()>>,
    receiver: mpsc::Receiver<WorkResult>,
    next_worker: usize,
}

impl WorkerPool {
    pub fn new(num_workers: usize, seed: u32, cache: ChunkCache) -> Self {
        let mut senders: Vec<mpsc::Sender<WorkItem>> = Vec::with_capacity(num_workers);
        let mut handles = Vec::with_capacity(num_workers);
        let (result_sender, receiver) = mpsc::channel();
        let side = 32;

        for _ in 0..num_workers {
            let (tx, rx) = mpsc::channel();
            senders.push(tx);

            let result_sender = result_sender.clone();
            let cache_clone = cache.clone();
            let handle = thread::spawn(move || {
                let generator = Generator::new(seed);
                while let Ok(item) = rx.recv() {
                    ///////// BADDD
                    let get = |cx: i32, cy: i32, cz: i32| -> Arc<[Block; CHUNK_VOLUME]> {
                        {
                            let map = cache_clone.lock().unwrap();
                            if let Some(b) = map.get(&(cx, cy, cz)) {
                                return b.clone();
                            }
                        }
                        let blocks = Arc::new(generator.generate_blocks(cx * side, cy * side, cz * side));
                        let mut map = cache_clone.lock().unwrap();
                        map.entry((cx, cy, cz)).or_insert(blocks).clone()
                    };

                    let cx = item.coords.0;
                    let cy = item.coords.1;
                    let cz = item.coords.2;

                    let blocks = get(cx, cy, cz);
                    let xp = get(cx + 1, cy, cz);
                    let xn = get(cx - 1, cy, cz);
                    let yp = get(cx, cy + 1, cz);
                    let yn = get(cx, cy - 1, cz);
                    let zp = get(cx, cy, cz + 1);
                    let zn = get(cx, cy, cz - 1);

                    let chunk_mesh = mesh(
                        &blocks,
                        Neighbours {
                            xp: Some(&xp),
                            xn: Some(&xn),
                            yp: Some(&yp),
                            yn: Some(&yn),
                            zp: Some(&zp),
                            zn: Some(&zn),
                        },
                    );
                    if result_sender.send(WorkResult { coords: item.coords, mesh: chunk_mesh }).is_err() {
                        break;
                    }
                }
            });
            handles.push(handle);
        }

        return WorkerPool { senders, handles, receiver, next_worker: 0 };
    }

    pub fn submit(&mut self, item: WorkItem) {
        if let Some(sender) = self.senders.get(self.next_worker) {
            let _ = sender.send(item);
            self.next_worker = (self.next_worker + 1) % self.senders.len();
        }
    }

    pub fn try_recv(&self) -> Option<WorkResult> {
        return self.receiver.try_recv().ok();
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.senders.clear();
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}
