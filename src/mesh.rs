use super::blocks::Blocks;

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
    pub const fn pack(x: u32, y: u32, z: u32, normal: u32, block_id: Blocks) -> Self {
        FaceData {
            data: ((x & 0xF) << 28)
                | ((y & 0xF) << 24)
                | ((z & 0xF) << 20)
                | ((normal & 0xFF) << 12)
                | ((block_id as u32 & 0xFF) << 8),
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
    FaceData::pack(1, 0, 0, 0, Blocks::Stone),
    // -X
    FaceData::pack(0, 0, 0, 1, Blocks::Stone),
    // +Y
    FaceData::pack(0, 1, 0, 2, Blocks::Stone),
    // -Y
    FaceData::pack(0, 0, 0, 3, Blocks::Stone),
    // +Z
    FaceData::pack(0, 0, 1, 4, Blocks::Stone),
    // -Z
    FaceData::pack(0, 0, 0, 5, Blocks::Stone),
];
