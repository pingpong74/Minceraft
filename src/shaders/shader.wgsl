struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct FaceInstanceInput {
    @location(0) data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};
const offsets: array<array<vec3<f32>, 6>, 6> = array(
    // +X face
    array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 1.0, 1.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 1.0),
        vec3<f32>(0.0, 0.0, 1.0)
    ),
    // -X face
    array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
        vec3<f32>(0.0, 1.0, 1.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 1.0),
        vec3<f32>(0.0, 1.0, 0.0)
    ),
    // +Y face
    array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 1.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 1.0),
        vec3<f32>(0.0, 0.0, 1.0)
    ),
    // -Y face
    array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
        vec3<f32>(1.0, 0.0, 1.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 1.0),
        vec3<f32>(1.0, 0.0, 0.0)
    ),
    // +Z face
    array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0)
    ),
    // -Z face
    array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0)
    )
);

fn unpack_position(data: u32) -> vec3<f32> {
    let x = f32((data >> 28u) & 0xFu);
    let y = f32((data >> 24u) & 0xFu);
    let z = f32((data >> 20u) & 0xFu);
    return vec3<f32>(x, y, z);
}

fn unpack_normal(data: u32) -> u32 {
    return (data >> 12u) & 0xFFu;
}

fn unpack_block_id(data: u32) -> u32 {
    return (data >> 8u) & 0xFu;
}

@vertex
fn vs_main(face_instance: FaceInstanceInput, @builtin(instance_index) instance_id: u32, @builtin(vertex_index) vertex_id: u32) -> VertexOutput {
    let normal = unpack_normal(face_instance.data);
    let pos = unpack_position(face_instance.data) + offsets[normal][vertex_id];

    var out: VertexOutput;
    out.color = pos;
    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
