use shaders::{Face, FaceRange};

use crate::block::{AIR, BlockId};
use crate::chunk::{CHUNK_SIDE, Chunk, Voxels};

pub struct Neighbours<'a> {
    pub xp: Option<&'a Chunk>,
    pub xn: Option<&'a Chunk>,
    pub yp: Option<&'a Chunk>,
    pub yn: Option<&'a Chunk>,
    pub zp: Option<&'a Chunk>,
    pub zn: Option<&'a Chunk>,
}

impl<'a> Neighbours<'a> {
    fn get(&self, chunk: &Chunk, x: i32, y: i32, z: i32) -> BlockId {
        let side = CHUNK_SIDE as i32;

        if (0..side).contains(&x) && (0..side).contains(&y) && (0..side).contains(&z) {
            return chunk.get(x as usize, y as usize, z as usize);
        }

        let (neighbour, lx, ly, lz) = if x >= side {
            (self.xp, x - side, y, z)
        } else if x < 0 {
            (self.xn, x + side, y, z)
        } else if y >= side {
            (self.yp, x, y - side, z)
        } else if y < 0 {
            (self.yn, x, y + side, z)
        } else if z >= side {
            (self.zp, x, y, z - side)
        } else {
            (self.zn, x, y, z + side)
        };

        match neighbour {
            Some(n) => n.get(lx as usize, ly as usize, lz as usize),
            None => AIR,
        }
    }
}

const FACE_NEIGHBORS: [(i32, i32, i32); 6] = [
    (1, 0, 0),  // +X
    (-1, 0, 0), // -X
    (0, 1, 0),  // +Y
    (0, -1, 0), // -Y
    (0, 0, 1),  // +Z
    (0, 0, -1), // -Z
];

pub struct ChunkMesh {
    pub quads: Vec<Face>,
    pub face_ranges: [FaceRange; 6],
}

pub fn mesh_chunk(chunk: &Chunk, neighbours: &Neighbours) -> ChunkMesh {
    if let Voxels::Uniform(block) = *chunk.voxels {
        if block == AIR {
            return ChunkMesh {
                quads: Vec::new(),
                face_ranges: [FaceRange { first_quad: 0, quad_count: 0 }; 6],
            };
        }
    }

    let mut faces: [Vec<Face>; 6] = std::array::from_fn(|_| Vec::new());

    for y in 0..CHUNK_SIDE {
        for z in 0..CHUNK_SIDE {
            for x in 0..CHUNK_SIDE {
                let block = chunk.get(x, y, z);
                if block == AIR {
                    continue;
                }

                for (face, &(dx, dy, dz)) in FACE_NEIGHBORS.iter().enumerate() {
                    if neighbours.get(chunk, x as i32 + dx, y as i32 + dy, z as i32 + dz) != AIR {
                        continue;
                    }

                    faces[face].push(Face::pack(x as u32, y as u32, z as u32, 1, 1, face as u32, block as u32, 0, 0, 0));
                }
            }
        }
    }

    let total_quads = faces.iter().map(Vec::len).sum();
    let mut quads = Vec::with_capacity(total_quads);
    let mut face_ranges = [FaceRange { first_quad: 0, quad_count: 0 }; 6];

    for (face, face_quads) in faces.into_iter().enumerate() {
        face_ranges[face] = FaceRange {
            first_quad: quads.len() as u32,
            quad_count: face_quads.len() as u32,
        };
        quads.extend(face_quads);
    }

    ChunkMesh { quads, face_ranges }
}
