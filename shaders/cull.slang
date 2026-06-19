import sgpu;

struct PushConstants {
    float4x4 view_proj;
    uint buffer_handle;
    uint face_count;
    uint chunk_info_handle;
    uint indirect_handle;
    uint count_handle;
    uint chunk_count;
    uint2 _pad;
};

[[vk_push_constant]] PushConstants pc;

struct ChunkInfo {
    float4 min;
    float4 max;
    uint face_offset;
    uint face_count;
    uint2 _pad;
};

struct DrawIndirectCommand {
    uint vertex_count;
    uint instance_count;
    uint first_vertex;
    uint first_instance;
};

float4 normalize_plane(float4 row) {
    float len = length(row.xyz);
    if (len < 1e-8) return float4(0, 0, 0, 0);
    return row / len;
}

bool aabb_visible(float3 center, float3 half_size, float4 plane) {
    float3 n = plane.xyz;
    float d = plane.w;
    float radius = dot(abs(n), half_size);
    float dist = dot(n, center) + d;
    return dist + radius >= 0.0;
}

[numthreads(64, 1, 1)]
void main(uint3 id : SV_DispatchThreadID) {
    uint chunk_index = id.x;
    if (chunk_index >= pc.chunk_count) return;

    float4 m0 = pc.view_proj[0];
    float4 m1 = pc.view_proj[1];
    float4 m2 = pc.view_proj[2];
    float4 m3 = pc.view_proj[3];

    float4 row0 = float4(m0.x, m1.x, m2.x, m3.x);
    float4 row1 = float4(m0.y, m1.y, m2.y, m3.y);
    float4 row2 = float4(m0.z, m1.z, m2.z, m3.z);
    float4 row3 = float4(m0.w, m1.w, m2.w, m3.w);

    float4 planes[6] = {
        normalize_plane(row3 + row0),
        normalize_plane(row3 - row0),
        normalize_plane(row3 + row1),
        normalize_plane(row3 - row1),
        normalize_plane(row2),
        normalize_plane(row3 - row2),
    };

    ReadOnlyBuffer<uint> info_buf = get_buffer<uint>(pc.chunk_info_handle);

    uint stride = 12;
    uint base = chunk_index * stride;

    float3 min_world = float3(
        asfloat(info_buf.load_indexed(base + 0)),
        asfloat(info_buf.load_indexed(base + 1)),
        asfloat(info_buf.load_indexed(base + 2))
    );
    float3 max_world = float3(
        asfloat(info_buf.load_indexed(base + 4)),
        asfloat(info_buf.load_indexed(base + 5)),
        asfloat(info_buf.load_indexed(base + 6))
    );

    float3 center = (min_world + max_world) * 0.5;
    float3 half_size = (max_world - min_world) * 0.5;

    for (int i = 0; i < 6; i++) {
        if (!aabb_visible(center, half_size, planes[i])) {
            return;
        }
    }

    uint f_offset = info_buf.load_indexed(base + 8);
    uint f_count = info_buf.load_indexed(base + 9);

    if (f_count == 0) return;

    ReadWriteBuffer<uint> indirect_buf = get_rw_buffer<uint>(pc.indirect_handle);
    ReadWriteBuffer<uint> count_buf = get_rw_buffer<uint>(pc.count_handle);

    uint idx = count_buf.interlocked_add(0, 1);

    uint cmd = idx * 4;
    indirect_buf.store_indexed(f_count * 6, cmd + 0);
    indirect_buf.store_indexed(1, cmd + 1);
    indirect_buf.store_indexed(f_offset * 6, cmd + 2);
    indirect_buf.store_indexed(0, cmd + 3);
}
