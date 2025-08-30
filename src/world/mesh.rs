use crate::world::blocks::ALL_BLOCKS;
use crate::world::blocks::BlockType;

use super::blocks::Block;
use super::blocks::FaceType;
use cgmath::Point3;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
//Layout
//      x       y       z       normal      BlockID
//      4       4       4         8            8    = 28 bits
//
// Normals:
// 0 => +x
// 1 => -x
// 2 => +y
// 3 => -y
// 4 => +z
// 5 => -z
pub struct FaceData {
    data: u32,
}

impl FaceData {
    pub const fn pack(x: usize, y: usize, z: usize, normal: u32, face_id: FaceType) -> Self {
        FaceData {
            data: ((x as u32 & 0xF) << 28)
                | ((y as u32 & 0xF) << 24)
                | ((z as u32 & 0xF) << 20)
                | ((normal & 0xFF) << 12)
                | ((face_id as u32 & 0xFF) << 8),
        }
    }

    pub fn get_vertex_descriptor() -> wgpu::VertexBufferLayout<'static> {
        return wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FaceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Uint32,
            }],
        };
    }
}

pub const FACES: &[FaceData] = &[
    // +X
    FaceData::pack(1, 0, 0, 0, FaceType::Dirt),
    // -X
    FaceData::pack(0, 0, 0, 1, FaceType::Dirt),
    // +Y
    FaceData::pack(0, 1, 0, 2, FaceType::Dirt),
    // -Y
    FaceData::pack(0, 0, 0, 3, FaceType::Dirt),
    // +Z
    FaceData::pack(0, 0, 1, 4, FaceType::Dirt),
    // -Z
    FaceData::pack(0, 0, 0, 5, FaceType::Dirt),
];

pub const CHUNK_SIZE: usize = 16;

pub struct Chunk {
    pub pos: cgmath::Point3<i32>,
    pub blocks: [[[BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub face: Vec<FaceData>,
}

impl Chunk {
    pub fn new(pos: Point3<i32>) -> Self {
        return Chunk {
            pos: pos,
            blocks: [[[BlockType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            face: Vec::new(),
        };
    }

    pub async fn generate(chunk: &mut Chunk) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if x * x + y * y + z * z < 64 {
                        chunk.blocks[x][y][z] = BlockType::Moss;
                    } else {
                        chunk.blocks[x][y][z] = BlockType::Air;
                    }
                }
            }
        }
    }

    // Normals:
    // 0 => +x
    // 1 => -x
    // 2 => +y
    // 3 => -y
    // 4 => +z
    // 5 => -z
    pub async fn mesh(chunk: &mut Chunk) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let curr_block = chunk.blocks[x][y][z].clone() as usize;

                    if curr_block == 0 {
                        break;
                    }

                    if y < CHUNK_SIZE - 1 && (chunk.blocks[x][y + 1][z] == BlockType::Air) {
                        chunk.face.push(FaceData::pack(
                            x,
                            y + 1,
                            z,
                            2,
                            ALL_BLOCKS[curr_block].face.clone(),
                        ));
                    }

                    if y > 0 && (chunk.blocks[x][y - 1][z] == BlockType::Air) {
                        chunk.face.push(FaceData::pack(
                            x,
                            y,
                            z,
                            3,
                            ALL_BLOCKS[curr_block].face.clone(),
                        ));
                    }
                }
            }
        }
    }
}
