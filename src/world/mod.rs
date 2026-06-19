mod worker_pool;

pub use worker_pool::*;

use crate::renderer::BufferLocation;
use std::collections::HashMap;

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
    generation_radius: u32,
    unload_radius: u32,
}

impl World {
    pub fn new(generation_radius: u32, unload_radius: u32) -> Self {
        World {
            chunks: HashMap::new(),
            generation_radius,
            unload_radius,
        }
    }

    pub fn update(&mut self, cx: i32, cy: i32, cz: i32) -> (Vec<(i32, i32, i32)>, Vec<ChunkUnloadInfo>) {
        let r_gen = self.generation_radius as i32;
        let r_unload = self.unload_radius as i32;
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
                    dx * dx + dy * dy + dz * dz > r_unload * r_unload
                }
            })
            .map(|(&key, _)| key)
            .collect();

        for key in unload_keys {
            if let Some(entry) = self.chunks.remove(&key) {
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
                    dx * dx + dy * dy + dz * dz > r_unload * r_unload
                }
            })
            .map(|(&key, _)| key)
            .collect();

        for key in empty_unload {
            self.chunks.remove(&key);
        }

        for dz in -r_gen..=r_gen {
            for dx in -r_gen..=r_gen {
                for dy in -r_gen..=r_gen {
                    if dx * dx + dz * dz + dy * dy > r_gen * r_gen {
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

        (to_load, to_unload)
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

    pub fn is_loaded(&self, coords: &(i32, i32, i32)) -> bool {
        self.chunks.get(coords).map_or(false, |e| e.state == ChunkState::Loaded)
    }

    pub fn is_pending(&self, coords: &(i32, i32, i32)) -> bool {
        self.chunks.get(coords).map_or(false, |e| e.state == ChunkState::Pending)
    }

    pub fn state(&self, coords: &(i32, i32, i32)) -> Option<ChunkState> {
        self.chunks.get(coords).map(|e| e.state)
    }

    pub fn loaded_chunk_count(&self) -> u32 {
        self.chunks.values().filter(|e| e.state == ChunkState::Loaded).count() as u32
    }

    pub fn active_slot_count(&self) -> u32 {
        self.chunks.values().filter(|e| e.state == ChunkState::Loaded && e.cmd_slot.is_some()).count() as u32
    }
}
