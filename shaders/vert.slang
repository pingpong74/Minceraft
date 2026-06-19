import sgpu;

struct PushConstants {
  float4x4 view_proj;
  uint face_buffer_id;
  uint indirect_draw_buffer_id;
};

[[vk_push_constant]]
PushConstants pc;

struct IndirectDrawCommand {
  uint vertex_count;
  uint instance_count;
  uint first_vertex;
  uint first_instance;
  int3 world_pos;
  uint _pad;
};

struct UnpackedFace {
  float3 pos;
  uint normal_idx;
  uint block_id;

  static UnpackedFace unpack(uint data) {
    float3 pos;
    pos.x = float(data & 0x1F);
    pos.y = float((data >> 5) & 0x1F);
    pos.z = float((data >> 10) & 0x1F);

    uint normal_idx = (data >> 15) & 0x7;
    uint block_id = (data >> 18) & 0xFFFF;

    return { pos, normal_idx, block_id };
  }
};

static const float3 corner_offsets[6][4] = {
  { float3(1, 0, 0), float3(1, 1, 0), float3(1, 1, 1), float3(1, 0, 1) },
  { float3(0, 0, 1), float3(0, 1, 1), float3(0, 1, 0), float3(0, 0, 0) },
  { float3(0, 1, 0), float3(0, 1, 1), float3(1, 1, 1), float3(1, 1, 0) },
  { float3(0, 0, 1), float3(0, 0, 0), float3(1, 0, 0), float3(1, 0, 1) },
  { float3(0, 0, 1), float3(1, 0, 1), float3(1, 1, 1), float3(0, 1, 1) },
  { float3(0, 0, 0), float3(0, 1, 0), float3(1, 1, 0), float3(1, 0, 0) },
};

static const int quad_indices[6] = { 0, 1, 2, 0, 2, 3 };

static const float4 block_colors[5] = {
  float4(0.5, 0.7, 0.3, 1.0), float4(0.6, 0.4, 0.2, 1.0),
  float4(0.3, 0.3, 0.8, 1.0), float4(0.9, 0.9, 0.2, 1.0),
  float4(0.8, 0.8, 0.8, 1.0),
};

static const float3 face_normals[6] = {
  float3(1, 0, 0),  float3(-1, 0, 0), float3(0, 1, 0),
  float3(0, -1, 0), float3(0, 0, 1),  float3(0, 0, -1),
};

struct VSOutput {
  float4 position : SV_Position;
  float4 color : COLOR0;
  float3 normal : NORMAL0;
};

VSOutput main(uint vertex_id: SV_VertexID, uint draw_id: SV_DrawIndex) {
  // extract the world positions from the indirect draw buffer
  ReadOnlyBuffer<IndirectDrawCommand> indirect_buf =
      get_buffer<IndirectDrawCommand>(pc.indirect_draw_buffer_id);

  IndirectDrawCommand draw_info = indirect_buf[draw_id];
  float3 world_pos = float3(32 * draw_info.world_pos);

  uint face_index = (vertex_id + draw_info.first_vertex) / 6;
  uint vertex_in_face = (vertex_id + draw_info.first_vertex) % 6;

  // unpack the face into components
  ReadOnlyBuffer<uint> face_buffer = get_buffer<uint>(pc.face_buffer_id);
  uint face_data = face_buffer[face_index];

  UnpackedFace face = UnpackedFace::unpack(face_data);

  // get the veretx position
  float3 vertex_pos =
      face.pos + corner_offsets[face.normal_idx][quad_indices[vertex_in_face]];

  VSOutput output;
  output.position = mul(pc.view_proj, float4(vertex_pos + world_pos, 1.0));
  output.color = block_colors[min(face.block_id - 1, 4)];
  output.normal = face_normals[face.normal_idx];
  return output;
}
