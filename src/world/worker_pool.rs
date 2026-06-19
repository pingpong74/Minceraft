use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use crate::chunk::{CHUNK_SIDE, ChunkMesh, Generator, Neighbours, mesh};

pub struct WorkItem {
    pub coords: (i32, i32, i32),
    pub world_x: i32,
    pub world_y: i32,
    pub world_z: i32,
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
    pub fn new(num_workers: usize, seed: u32) -> Self {
        let mut senders: Vec<mpsc::Sender<WorkItem>> = Vec::with_capacity(num_workers);
        let mut handles = Vec::with_capacity(num_workers);
        let (result_sender, receiver) = mpsc::channel();
        let side = CHUNK_SIDE as i32;

        for _ in 0..num_workers {
            let (tx, rx) = mpsc::channel();
            senders.push(tx);

            let result_sender = result_sender.clone();
            let handle = thread::spawn(move || {
                let generator = Generator::new(seed);
                while let Ok(item) = rx.recv() {
                    let blocks = generator.generate_blocks(item.world_x, item.world_y, item.world_z);

                    let xp = generator.generate_blocks(item.world_x + side, item.world_y, item.world_z);
                    let xn = generator.generate_blocks(item.world_x - side, item.world_y, item.world_z);
                    let yp = generator.generate_blocks(item.world_x, item.world_y + side, item.world_z);
                    let yn = generator.generate_blocks(item.world_x, item.world_y - side, item.world_z);
                    let zp = generator.generate_blocks(item.world_x, item.world_y, item.world_z + side);
                    let zn = generator.generate_blocks(item.world_x, item.world_y, item.world_z - side);

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

        WorkerPool { senders, handles, receiver, next_worker: 0 }
    }

    pub fn submit(&mut self, item: WorkItem) {
        if let Some(sender) = self.senders.get(self.next_worker) {
            let _ = sender.send(item);
            self.next_worker = (self.next_worker + 1) % self.senders.len();
        }
    }

    pub fn try_recv(&self) -> Option<WorkResult> {
        self.receiver.try_recv().ok()
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
