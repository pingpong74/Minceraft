use crate::block::BlockId;
use shaders::{ChunkMeta, FaceRange};
use std::sync::Arc;

pub const CHUNK_SIDE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkKey {
    pub location: glam::IVec3,
}

impl ChunkKey {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { location: glam::IVec3::new(x, y, z) }
    }
}

pub enum Voxels {
    Uniform(BlockId),
    Paletted { pallete: Vec<BlockId>, bits_per_voxel: u8, blocks: Vec<u64> },
}

impl Voxels {
    pub fn from_fn(mut f: impl FnMut(usize, usize, usize) -> BlockId) -> Self {
        let mut raw = [0u16; CHUNK_VOLUME];
        let mut first = None;
        let mut is_uniform = true;

        for z in 0..CHUNK_SIDE {
            for y in 0..CHUNK_SIDE {
                for x in 0..CHUNK_SIDE {
                    let b = f(x, y, z);
                    let idx = x + y * CHUNK_SIDE + z * CHUNK_SIDE * CHUNK_SIDE;
                    raw[idx] = b;
                    if let Some(prev) = first {
                        if prev != b {
                            is_uniform = false;
                        }
                    } else {
                        first = Some(b);
                    }
                }
            }
        }

        if is_uniform {
            return Self::Uniform(first.unwrap_or(0));
        }

        let mut pallete = Vec::new();
        for &b in &raw {
            if !pallete.contains(&b) {
                pallete.push(b);
            }
        }

        let bits_per_voxel = pallete.len().next_power_of_two().trailing_zeros().max(1) as u8;
        let total_bits = CHUNK_VOLUME * bits_per_voxel as usize;
        let word_count = (total_bits + 63) / 64;
        let mut blocks = vec![0u64; word_count + 1];

        for (voxel, &b) in raw.iter().enumerate() {
            let p_idx = pallete.iter().position(|&x| x == b).unwrap() as u64;
            let bit_offset = voxel * bits_per_voxel as usize;
            let word_index = bit_offset / 64;
            let shift = bit_offset % 64;

            blocks[word_index] |= p_idx << shift;
            if shift + bits_per_voxel as usize > 64 {
                blocks[word_index + 1] |= p_idx >> (64 - shift);
            }
        }

        Self::Paletted { pallete, bits_per_voxel, blocks }
    }

    pub fn block_at(&self, x: usize, y: usize, z: usize) -> BlockId {
        match self {
            Self::Uniform(u) => *u,
            Self::Paletted { pallete, bits_per_voxel, blocks } => {
                let bits = *bits_per_voxel as usize;
                let voxel = x + y * CHUNK_SIDE + z * CHUNK_SIDE * CHUNK_SIDE;

                let bit_offset = voxel * bits;
                let word_index = bit_offset / 64;
                let shift = bit_offset % 64;

                let mut value = blocks[word_index] >> shift;

                if shift + bits > 64 {
                    value |= blocks[word_index + 1] << (64 - shift);
                }

                pallete[(value & ((1u64 << bits) - 1)) as usize]
            }
        }
    }
}

pub struct Chunk {
    pub key: ChunkKey,
    pub voxels: Arc<Voxels>,
}

impl Chunk {
    pub fn new(key: ChunkKey, voxels: Voxels) -> Self {
        Self { key, voxels: Arc::new(voxels) }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.voxels.block_at(x, y, z)
    }

    pub fn meta(&self, face_ranges: [FaceRange; 6]) -> ChunkMeta {
        let side = CHUNK_SIDE as f32;
        let origin = [self.key.location.x as f32 * side, self.key.location.y as f32 * side, self.key.location.z as f32 * side, 0.0];

        ChunkMeta {
            origin,
            aabb_min: origin,
            aabb_max: [origin[0] + side, origin[1] + side, origin[2] + side, 0.0],
            face_ranges,
        }
    }

    pub fn get_or_air(&self, x: i32, y: i32, z: i32) -> BlockId {
        if x < 0 || y < 0 || z < 0 || x >= CHUNK_SIDE as i32 || y >= CHUNK_SIDE as i32 || z >= CHUNK_SIDE as i32 {
            crate::block::AIR
        } else {
            self.get(x as usize, y as usize, z as usize)
        }
    }
}
