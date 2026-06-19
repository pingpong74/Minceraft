module common;

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
