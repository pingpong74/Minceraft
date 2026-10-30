use sgpu_shaders::glam::{Vec2, Vec4};
use sgpu_shaders::*;

use crate::{ChunkIndirectDrawCommand, Face};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MeshPushConstants {
    pub view_proj: [f32; 16],
    pub chunk_draw_cmd_buffer: BufferId<Read>,
    pub quad_buffer: BufferId<Read>,
    pub texture: SampledImageId,
    pub sampler: SamplerId,
}

const CORNERS: [[[f32; 3]; 4]; 6] = [
    // +X
    [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]],
    // -X
    [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0]],
    // +Y
    [[0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    // -Y
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    // +Z
    [[1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
    // -Z
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
];

const TRIANGLE_CORNERS: [u32; 6] = [0, 1, 2, 0, 2, 3];

const UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

#[spirv(vertex)]
pub fn mesh_vs(#[spirv(draw_index)] draw_index: u32, #[spirv(vertex_index)] vertex_index: u32, #[spirv(position)] out_position: &mut Vec4, out_uv: &mut Vec2, #[spirv(push_constant)] push: &MeshPushConstants) {
    let face_buffer = ReadOnlyBuffer::<Face>::get(push.quad_buffer);
    let face = face_buffer.load_indexed(vertex_index / 6).unpack();

    let corner_index = TRIANGLE_CORNERS[(vertex_index % 6) as usize] as usize;
    let corner = CORNERS[face.face as usize][corner_index];

    let chunk_draw_cmd_buffer = ReadOnlyBuffer::<ChunkIndirectDrawCommand>::get(push.chunk_draw_cmd_buffer);
    let chunk_origin = chunk_draw_cmd_buffer.load_indexed(draw_index).world_position;

    *out_position = glam::Mat4::from_cols_array(&push.view_proj) * Vec4::new(face.rel_pos.x + corner[0] + chunk_origin[0], face.rel_pos.y + corner[1] + chunk_origin[1], face.rel_pos.z + corner[2] + chunk_origin[2], 1.0);
    *out_uv = Vec2::from_array(UVS[corner_index]);
}

#[spirv(fragment)]
pub fn mesh_fs(in_uv: Vec2, #[spirv(push_constant)] push: &MeshPushConstants, output: &mut Vec4) {
    let image = get_sampled_2d_f(push.texture);
    let sampler = SamplerId::get_sampler(push.sampler);
    *output = image.sample(*sampler, in_uv);
}
