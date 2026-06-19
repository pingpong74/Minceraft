use super::{Block, Face, chunk::*};

pub struct Neighbours<'a> {
    pub xp: Option<&'a [Block; CHUNK_VOLUME]>,
    pub xn: Option<&'a [Block; CHUNK_VOLUME]>,
    pub yp: Option<&'a [Block; CHUNK_VOLUME]>,
    pub yn: Option<&'a [Block; CHUNK_VOLUME]>,
    pub zp: Option<&'a [Block; CHUNK_VOLUME]>,
    pub zn: Option<&'a [Block; CHUNK_VOLUME]>,
}

#[inline]
fn get_block(center: &[Block; CHUNK_VOLUME], neigh: &Neighbours, x: i32, y: i32, z: i32) -> Block {
    if x >= 0 && x < CHUNK_SIDE as i32 && y >= 0 && y < CHUNK_SIDE as i32 && z >= 0 && z < CHUNK_SIDE as i32 {
        return center[Chunk::get_index(x as usize, y as usize, z as usize)];
    }

    let res = if x < 0 {
        neigh.xn.map(|c| c[Chunk::get_index(CHUNK_SIDE - 1, y as usize, z as usize)])
    } else if x >= CHUNK_SIDE as i32 {
        neigh.xp.map(|c| c[Chunk::get_index(0, y as usize, z as usize)])
    } else if y < 0 {
        neigh.yn.map(|c| c[Chunk::get_index(x as usize, CHUNK_SIDE - 1, z as usize)])
    } else if y >= CHUNK_SIDE as i32 {
        neigh.yp.map(|c| c[Chunk::get_index(x as usize, 0, z as usize)])
    } else if z < 0 {
        neigh.zn.map(|c| c[Chunk::get_index(x as usize, y as usize, CHUNK_SIDE - 1)])
    } else if z >= CHUNK_SIDE as i32 {
        neigh.zp.map(|c| c[Chunk::get_index(x as usize, y as usize, 0)])
    } else {
        None
    };

    return res.unwrap_or(Block::AIR);
}
const FACES: [(i32, i32, i32); 6] = [
    (1, 0, 0),  // +X
    (-1, 0, 0), // -X
    (0, 1, 0),  // +Y
    (0, -1, 0), // -Y
    (0, 0, 1),  // +Z
    (0, 0, -1), // -Z
];

// 0 -> +X
// 1 -> -X
// 2 -> +Y
// 3 -> -Y
// 4 -> +z
// 5 -> -Z

pub fn mesh(blocks: &[Block; CHUNK_VOLUME], neigh: Neighbours) -> ChunkMesh {
    let mut vertices: Vec<Face> = Vec::new();

    for z in 0..CHUNK_SIDE {
        for y in 0..CHUNK_SIDE {
            for x in 0..CHUNK_SIDE {
                let mat = blocks[Chunk::get_index(x, y, z)];

                if mat.is_air() {
                    continue;
                }

                for (i, (dx, dy, dz)) in FACES.iter().enumerate() {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let nz = z as i32 + dz;

                    let neighbor_mat = get_block(blocks, &neigh, nx, ny, nz);

                    if neighbor_mat.is_air() {
                        vertices.push(Face::new(x as u32, y as u32, z as u32, i as u32, mat));
                    }
                }
            }
        }
    }

    return ChunkMesh { faces: vertices };
}
