use super::{Block, Face};

pub struct ChunkMesh {
    pub faces: Vec<Face>,
}

pub const CHUNK_SIDE: usize = 32;
pub const CHUNK_VOLUME: usize = 32 * 32 * 32;

pub struct Chunk {
    pub blocks: [Block; CHUNK_VOLUME],
}

impl Chunk {
    #[inline]
    pub const fn get_index(x: usize, y: usize, z: usize) -> usize {
        return x + y * CHUNK_SIDE + z * CHUNK_SIDE * CHUNK_SIDE;
    }

    pub fn new(blocks: [Block; CHUNK_VOLUME]) -> Self {
        Chunk { blocks }
    }
}
