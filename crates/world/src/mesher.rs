use shaders::{Face, FaceRange};

use crate::block::{is_opaque, texture_layer};
use crate::chunk::{CHUNK_SIDE, Chunk, Voxels};

fn to_xyz(axis: usize, layer: usize, u: usize, v: usize) -> [usize; 3] {
    let mut p = [0; 3];
    p[axis] = layer;
    p[(axis + 1) % 3] = u;
    p[(axis + 2) % 3] = v;
    p
}

pub enum BorderMask {
    Air,
    Solid,
    Rows(Box<[u32; CHUNK_SIDE]>),
}

pub struct Border {
    pub mask: BorderMask,
    pub skirt: bool,
}

impl Border {
    pub const AIR: Border = Border { mask: BorderMask::Air, skirt: false };
    pub const SOLID: Border = Border { mask: BorderMask::Solid, skirt: false };

    pub fn from_chunk(neighbour: &Chunk, face: usize) -> Border {
        if let Voxels::Uniform(block) = *neighbour.voxels {
            return if is_opaque(block) { Border::SOLID } else { Border::AIR };
        }

        let axis = face / 2;
        let layer = if face % 2 == 0 { 0 } else { CHUNK_SIDE - 1 }; // the neighbour's layer that touches us
        let mut rows = Box::new([0u32; CHUNK_SIDE]);

        for v in 0..CHUNK_SIDE {
            for u in 0..CHUNK_SIDE {
                let p = to_xyz(axis, layer, u, v);
                if is_opaque(neighbour.get(p[0], p[1], p[2])) {
                    rows[v] |= 1 << u;
                }
            }
        }

        Border { mask: BorderMask::Rows(rows), skirt: false }
    }

    pub fn opaque_at(&self, u: usize, v: usize) -> bool {
        match &self.mask {
            BorderMask::Air => false,
            BorderMask::Solid => true,
            BorderMask::Rows(rows) => rows[v] >> u & 1 == 1,
        }
    }
}

/// One border per face of the chunk, in face order.
/// +X, -X, +Y, -Y, +Z, -Z
pub struct Borders(pub [Border; 6]);

impl Borders {
    pub fn air() -> Self {
        Self([Border::AIR; 6])
    }
}

pub struct ChunkMesh {
    pub quads: Vec<Face>,
    pub face_ranges: [FaceRange; 6],
}

/// Binary greedy meshing of the opaque blocks of one chunk.
pub fn mesh_chunk(chunk: &Chunk, borders: &Borders) -> ChunkMesh {
    let empty = ChunkMesh {
        quads: Vec::new(),
        face_ranges: [FaceRange { first_quad: 0, quad_count: 0 }; 6],
    };

    if let Voxels::Uniform(block) = *chunk.voxels {
        if !is_opaque(block) {
            return empty;
        }
    }

    let mut cols = [[0u64; CHUNK_SIDE * CHUNK_SIDE]; 3];
    for z in 0..CHUNK_SIDE {
        for y in 0..CHUNK_SIDE {
            for x in 0..CHUNK_SIDE {
                if is_opaque(chunk.voxels.block_at(x, y, z)) {
                    cols[0][z * CHUNK_SIDE + y] |= 1 << (x + 1);
                    cols[1][x * CHUNK_SIDE + z] |= 1 << (y + 1);
                    cols[2][y * CHUNK_SIDE + x] |= 1 << (z + 1);
                }
            }
        }
    }

    let mut planes = [[0u32; CHUNK_SIDE * CHUNK_SIDE]; 6];
    for axis in 0..3 {
        let (positive, negative) = (&borders.0[axis * 2], &borders.0[axis * 2 + 1]);

        for v in 0..CHUNK_SIDE {
            for u in 0..CHUNK_SIDE {
                let mut col = cols[axis][v * CHUNK_SIDE + u];
                if negative.opaque_at(u, v) {
                    col |= 1;
                }
                if positive.opaque_at(u, v) {
                    col |= 1 << (CHUNK_SIDE + 1);
                }

                let pos = (((col & !(col >> 1)) >> 1) & 0xFFFF_FFFF) as u32;
                let neg = (((col & !(col << 1)) >> 1) & 0xFFFF_FFFF) as u32;
                scatter(&mut planes[axis * 2], pos, u, v);
                scatter(&mut planes[axis * 2 + 1], neg, u, v);
            }
        }

        if positive.skirt {
            add_skirt(&mut planes[axis * 2], &cols[axis], true);
        }
        if negative.skirt {
            add_skirt(&mut planes[axis * 2 + 1], &cols[axis], false);
        }
    }

    let mut faces: [Vec<Face>; 6] = std::array::from_fn(|_| Vec::new());

    for face in 0..6 {
        let axis = face / 2;
        let plane = &mut planes[face];
        let texture_at = |layer: usize, u: usize, v: usize| {
            let p = to_xyz(axis, layer, u, v);
            texture_layer(chunk.voxels.block_at(p[0], p[1], p[2]), face)
        };

        for layer in 0..CHUNK_SIDE {
            for v in 0..CHUNK_SIDE {
                loop {
                    let bits = plane[layer * CHUNK_SIDE + v];
                    if bits == 0 {
                        break;
                    }

                    let u = bits.trailing_zeros() as usize;
                    let texture = texture_at(layer, u, v);

                    let run = (bits >> u).trailing_ones() as usize;
                    let mut w = 1;
                    while w < run && texture_at(layer, u + w, v) == texture {
                        w += 1;
                    }
                    let mask = (((1u64 << w) - 1) << u) as u32;

                    let mut h = 1;
                    while v + h < CHUNK_SIDE && plane[layer * CHUNK_SIDE + v + h] & mask == mask && (u..u + w).all(|x| texture_at(layer, x, v + h) == texture) {
                        h += 1;
                    }

                    for row in v..v + h {
                        plane[layer * CHUNK_SIDE + row] &= !mask;
                    }

                    let p = to_xyz(axis, layer, u, v);
                    faces[face].push(Face::pack(p[0] as u32, p[1] as u32, p[2] as u32, w as u32, h as u32, face as u32, texture as u32, 0, 0, 0));
                }
            }
        }
    }

    let mut quads = Vec::with_capacity(faces.iter().map(Vec::len).sum());
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

fn scatter(plane: &mut [u32; CHUNK_SIDE * CHUNK_SIDE], mut layers: u32, u: usize, v: usize) {
    while layers != 0 {
        let layer = layers.trailing_zeros() as usize;
        layers &= layers - 1;
        plane[layer * CHUNK_SIDE + v] |= 1 << u;
    }
}

fn add_skirt(plane: &mut [u32; CHUNK_SIDE * CHUNK_SIDE], cols: &[u64; CHUNK_SIDE * CHUNK_SIDE], positive: bool) {
    // voxel i is bit i + 1: boundary voxel 31 / 0, the two voxels behind it 30,29 / 1,2
    let (boundary_bit, behind, layer) = if positive { (CHUNK_SIDE, 0b11u64 << 30, CHUNK_SIDE - 1) } else { (1, 0b11u64 << 2, 0) };
    let air_behind = |u: usize, v: usize| cols[v * CHUNK_SIDE + u] & behind != behind;

    for v in 0..CHUNK_SIDE {
        for u in 0..CHUNK_SIDE {
            if cols[v * CHUNK_SIDE + u] >> boundary_bit & 1 == 0 {
                continue;
            }

            let near_air = air_behind(u, v) || (u > 0 && air_behind(u - 1, v)) || (u + 1 < CHUNK_SIDE && air_behind(u + 1, v)) || (v > 0 && air_behind(u, v - 1)) || (v + 1 < CHUNK_SIDE && air_behind(u, v + 1));
            if near_air {
                plane[layer * CHUNK_SIDE + v] |= 1 << u;
            }
        }
    }
}
