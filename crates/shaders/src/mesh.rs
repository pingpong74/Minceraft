use sgpu_shaders::glam::{Vec2, Vec3, Vec4};
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

// (u_axis, u_sign, u_offset, v_axis, v_sign, v_offset)
// value = sign * local[axis] + offset * size[axis]
const UV_AXES: [(usize, f32, f32, usize, f32, f32); 6] = [
    (2, -1.0, 1.0, 1, -1.0, 1.0), // +X
    (2, 1.0, 0.0, 1, -1.0, 1.0),  // -X
    (0, 1.0, 0.0, 2, 1.0, 0.0),   // +Y
    (0, 1.0, 0.0, 2, -1.0, 1.0),  // -Y
    (0, 1.0, 0.0, 1, -1.0, 1.0),  // +Z
    (0, -1.0, 1.0, 1, -1.0, 1.0), // -Z
];
#[inline]
pub fn quad_uv(face: usize, local: [f32; 3], size: [f32; 3]) -> [f32; 2] {
    let (ua, us, uo, va, vs, vo) = UV_AXES[face];
    [us * local[ua] + uo * size[ua], vs * local[va] + vo * size[va]]
}

const CORNERS: [[[f32; 3]; 4]; 6] = [
    // +X
    [[1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]],
    // -X
    [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
    // +Y
    [[0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    // -Y
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    // +Z
    [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
    // -Z
    [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
];

const TRIANGLE_CORNERS: [u32; 6] = [0, 1, 2, 0, 2, 3];

#[spirv(vertex)]
pub fn mesh_vs(#[spirv(draw_index)] draw_index: u32, #[spirv(vertex_index)] vertex_index: u32, #[spirv(position)] out_position: &mut Vec4, out_uv: &mut Vec2, #[spirv(flat)] out_layer: &mut u32, #[spirv(push_constant)] push: &MeshPushConstants) {
    let face_buffer = ReadOnlyBuffer::<Face>::get(push.quad_buffer);
    let face = face_buffer.load_indexed(vertex_index / 6).unpack();

    let face_idx = face.face as usize;
    let axis = face_idx / 2;
    let u_axis = (axis + 1) % 3;
    let v_axis = (axis + 2) % 3;

    let corner_index = TRIANGLE_CORNERS[(vertex_index % 6) as usize] as usize;
    let corner = CORNERS[face_idx][corner_index];

    let mut scales = [1.0f32; 3];
    scales[u_axis] = face.width as f32;
    scales[v_axis] = face.height as f32;

    let chunk_draw_cmd_buffer = ReadOnlyBuffer::<ChunkIndirectDrawCommand>::get(push.chunk_draw_cmd_buffer);
    let chunk_origin = chunk_draw_cmd_buffer.load_indexed(draw_index).world_position;

    // position of this corner inside the (merged) quad, in voxels
    let local = [corner[0] * scales[0], corner[1] * scales[1], corner[2] * scales[2]];

    let world = Vec4::new(face.rel_pos.x + local[0] + chunk_origin[0], face.rel_pos.y + local[1] + chunk_origin[1], face.rel_pos.z + local[2] + chunk_origin[2], 1.0);
    *out_position = glam::Mat4::from_cols_array(&push.view_proj) * world;

    let uv = quad_uv(face_idx, local, scales);

    *out_uv = Vec2::new(uv[0], uv[1]);
    *out_layer = face.block_id as u32;
}

#[spirv(fragment)]
pub fn mesh_fs(in_uv: Vec2, #[spirv(flat)] in_layer: u32, #[spirv(push_constant)] push: &MeshPushConstants, output: &mut Vec4) {
    let image = get_sampled_2d_array_f(push.texture);
    let sampler = SamplerId::get_sampler(push.sampler);
    *output = image.sample(*sampler, Vec3::new(in_uv.x, in_uv.y, in_layer as f32));
}
