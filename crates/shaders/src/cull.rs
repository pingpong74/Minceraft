use sgpu_shaders::{
    glam::{UVec3, Vec3},
    *,
};

use crate::{ChunkIndirectDrawCommand, ChunkMeta};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CullPushConstants {
    pub chunks: BufferId<Read>,
    pub draws: BufferId<Write>,
    pub counts: BufferId<Write>,
    pub chunk_count: u32,
    pub max_draws: u32,
    pub _pad: [u32; 2],
    pub frustum_planes: [[f32; 4]; 6],
    pub camera_pos: [f32; 4],
}

const VERTICES_PER_QUAD: u32 = 6;

#[inline]
fn aabb_inside_frustum(chunk: &ChunkMeta, planes: &[[f32; 4]; 6]) -> bool {
    let mut plane_index = 0u32;
    while plane_index < 6 {
        let plane = planes[plane_index as usize];
        let normal = Vec3::new(plane[0], plane[1], plane[2]);
        let positive = Vec3::new(
            if normal.x >= 0.0 { chunk.aabb_max[0] } else { chunk.aabb_min[0] },
            if normal.y >= 0.0 { chunk.aabb_max[1] } else { chunk.aabb_min[1] },
            if normal.z >= 0.0 { chunk.aabb_max[2] } else { chunk.aabb_min[2] },
        );

        if normal.dot(positive) + plane[3] < 0.0 {
            return false;
        }

        plane_index += 1;
    }

    true
}

#[inline]
fn face_visible(face: u32, camera: Vec3, chunk: &ChunkMeta) -> bool {
    match face {
        0 => camera.x > chunk.aabb_min[0], // +X
        1 => camera.x < chunk.aabb_max[0], // -X
        2 => camera.y > chunk.aabb_min[1], // +Y
        3 => camera.y < chunk.aabb_max[1], // -Y
        4 => camera.z > chunk.aabb_min[2], // +Z
        5 => camera.z < chunk.aabb_max[2], // -Z
        _ => false,
    }
}

#[spirv(compute(threads(64)))]
pub fn cull_cs(#[spirv(global_invocation_id)] id: UVec3, #[spirv(push_constant)] push: &CullPushConstants) {
    if id.x >= push.chunk_count {
        return;
    }

    let chunks = ReadOnlyBuffer::<ChunkMeta>::get(push.chunks);
    let chunk = chunks.load_indexed(id.x);

    if !aabb_inside_frustum(&chunk, &push.frustum_planes) {
        return;
    }

    let camera = Vec3::new(push.camera_pos[0], push.camera_pos[1], push.camera_pos[2]);
    let mut indirect_draw_buffer = ReadWriteBuffer::<ChunkIndirectDrawCommand>::get(push.draws);
    let mut count_buffer = ReadWriteBuffer::<u32>::get(push.counts);

    for i in 0..6 {
        let range = chunk.face_ranges[i as usize];

        if range.quad_count != 0 && face_visible(i, camera, &chunk) {
            let slot = count_buffer.interlocked_add(0, 1);

            if slot < push.max_draws {
                indirect_draw_buffer.store_indexed(
                    ChunkIndirectDrawCommand {
                        draw_command: DrawIndirectCommand {
                            vertex_count: range.quad_count * VERTICES_PER_QUAD,
                            instance_count: 1,
                            first_vertex: range.first_quad * VERTICES_PER_QUAD,
                            first_instance: 0,
                        },
                        world_position: chunk.aabb_min,
                    },
                    slot,
                );
            }
        }
    }
}
