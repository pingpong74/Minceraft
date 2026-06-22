mod worker_pool;

pub use worker_pool::*;

use crate::chunk::{Block, CHUNK_VOLUME};
use crate::renderer::BufferLocation;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
pub enum ChunkState {
    Pending,
    Loaded,
}

struct ChunkEntry {
    state: ChunkState,
    face_loc: Option<BufferLocation>,
    cmd_slot: Option<usize>,
}

pub struct ChunkUnloadInfo {
    pub coords: (i32, i32, i32),
    pub face_loc: BufferLocation,
    pub cmd_slot: usize,
}

pub struct World {
    chunks: HashMap<(i32, i32, i32), ChunkEntry>,
    generation_radius: i32,
    unload_radius: i32,
    chunk_cache: Arc<Mutex<HashMap<(i32, i32, i32), Arc<[Block; CHUNK_VOLUME]>>>>,
}

impl World {
    pub fn new(generation_radius: u32, unload_radius: u32) -> Self {
        return World {
            chunks: HashMap::new(),
            generation_radius: generation_radius as i32,
            unload_radius: unload_radius as i32,
            chunk_cache: Arc::new(Mutex::new(HashMap::new())),
        };
    }

    pub fn chunk_cache(&self) -> Arc<Mutex<HashMap<(i32, i32, i32), Arc<[Block; CHUNK_VOLUME]>>>> {
        self.chunk_cache.clone()
    }

    pub fn update(&mut self, cx: i32, cy: i32, cz: i32) -> (Vec<(i32, i32, i32)>, Vec<ChunkUnloadInfo>) {
        let mut to_load = Vec::new();
        let mut to_unload = Vec::new();

        let unload_keys: Vec<(i32, i32, i32)> = self
            .chunks
            .iter()
            .filter(|(key, entry)| {
                entry.state == ChunkState::Loaded && entry.face_loc.is_some() && {
                    let dx = key.0 - cx;
                    let dy = key.1 - cy;
                    let dz = key.2 - cz;
                    dx * dx + dy * dy + dz * dz > self.unload_radius * self.unload_radius
                }
            })
            .map(|(&key, _)| key)
            .collect();

        for key in unload_keys {
            if let Some(entry) = self.chunks.remove(&key) {
                self.chunk_cache.lock().unwrap().remove(&key);
                to_unload.push(ChunkUnloadInfo {
                    coords: key,
                    face_loc: entry.face_loc.unwrap(),
                    cmd_slot: entry.cmd_slot.unwrap(),
                });
            }
        }

        let empty_unload: Vec<(i32, i32, i32)> = self
            .chunks
            .iter()
            .filter(|(key, entry)| {
                entry.state == ChunkState::Loaded && entry.face_loc.is_none() && {
                    let dx = key.0 - cx;
                    let dy = key.1 - cy;
                    let dz = key.2 - cz;
                    dx * dx + dy * dy + dz * dz > self.unload_radius * self.unload_radius
                }
            })
            .map(|(&key, _)| key)
            .collect();

        for key in empty_unload {
            self.chunks.remove(&key);
            self.chunk_cache.lock().unwrap().remove(&key);
        }

        for dz in -self.generation_radius..=self.generation_radius {
            for dx in -self.generation_radius..=self.generation_radius {
                for dy in -self.generation_radius..=self.generation_radius {
                    if dx * dx + dz * dz + dy * dy > self.generation_radius * self.generation_radius {
                        continue;
                    }
                    let coords = (cx + dx, cy + dy, cz + dz);

                    if !self.chunks.contains_key(&coords) {
                        self.chunks.insert(
                            coords,
                            ChunkEntry {
                                state: ChunkState::Pending,
                                face_loc: None,
                                cmd_slot: None,
                            },
                        );
                        to_load.push(coords);
                    }
                }
            }
        }

        return (to_load, to_unload);
    }

    pub fn mark_loaded(&mut self, coords: (i32, i32, i32), face_loc: BufferLocation, cmd_slot: usize) {
        if let Some(entry) = self.chunks.get_mut(&coords) {
            entry.state = ChunkState::Loaded;
            entry.face_loc = Some(face_loc);
            entry.cmd_slot = Some(cmd_slot);
        }
    }

    pub fn mark_loaded_empty(&mut self, coords: (i32, i32, i32)) {
        if let Some(entry) = self.chunks.get_mut(&coords) {
            entry.state = ChunkState::Loaded;
        }
    }
}
