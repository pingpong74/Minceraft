module sgpu;

#define DECL_IMAGES(T, SUFFIX)                                                 \
  [[vk::binding(0, 0)]]                                                        \
  Texture1D<T> _s1d_##SUFFIX[];                                                \
  [[vk::binding(0, 0)]]                                                        \
  Texture2D<T> _s2d_##SUFFIX[];                                                \
  [[vk::binding(0, 0)]]                                                        \
  Texture3D<T> _s3d_##SUFFIX[];                                                \
  [[vk::binding(0, 0)]]                                                        \
  TextureCube<T> _scube_##SUFFIX[];                                            \
  [[vk::binding(1, 0)]]                                                        \
  RWTexture1D<T> _rw1d_##SUFFIX[];                                             \
  [[vk::binding(1, 0)]]                                                        \
  RWTexture2D<T> _rw2d_##SUFFIX[];                                             \
  [[vk::binding(1, 0)]]                                                        \
  RWTexture3D<T> _rw3d_##SUFFIX[];                                             \
                                                                               \
  public Texture1D<T> get_sampled_1d_##SUFFIX(uint h) {                        \
    return _s1d_##SUFFIX[NonUniformResourceIndex(h)];                          \
  }                                                                            \
  public Texture2D<T> get_sampled_2d_##SUFFIX(uint h) {                        \
    return _s2d_##SUFFIX[NonUniformResourceIndex(h)];                          \
  }                                                                            \
  public Texture3D<T> get_sampled_3d_##SUFFIX(uint h) {                        \
    return _s3d_##SUFFIX[NonUniformResourceIndex(h)];                          \
  }                                                                            \
  public TextureCube<T> get_sampled_cube_##SUFFIX(uint h) {                    \
    return _scube_##SUFFIX[NonUniformResourceIndex(h)];                        \
  }                                                                            \
  public RWTexture1D<T> get_rw_1d_##SUFFIX(uint h) {                           \
    return _rw1d_##SUFFIX[NonUniformResourceIndex(h)];                         \
  }                                                                            \
  public RWTexture2D<T> get_rw_2d_##SUFFIX(uint h) {                           \
    return _rw2d_##SUFFIX[NonUniformResourceIndex(h)];                         \
  }                                                                            \
  public RWTexture3D<T> get_rw_3d_##SUFFIX(uint h) {                           \
    return _rw3d_##SUFFIX[NonUniformResourceIndex(h)];                         \
  }

DECL_IMAGES(float, f)
DECL_IMAGES(float2, f2)
DECL_IMAGES(float3, f3)
DECL_IMAGES(float4, f4)
DECL_IMAGES(half, h)
DECL_IMAGES(half2, h2)
DECL_IMAGES(half3, h3)
DECL_IMAGES(half4, h4)
DECL_IMAGES(uint, u)
DECL_IMAGES(uint2, u2)
DECL_IMAGES(uint3, u3)
DECL_IMAGES(uint4, u4)
DECL_IMAGES(int, i)
DECL_IMAGES(int2, i2)
DECL_IMAGES(int3, i3)
DECL_IMAGES(int4, i4)

#undef DECL_IMAGES

// Buffers :

[[vk::binding(2, 0)]]
ByteAddressBuffer _buffers[];

[[vk::binding(2, 0)]]
RWByteAddressBuffer _rw_buffers[];

public struct ReadOnlyBuffer<T> {
  uint handle;

  public T load(uint byte_offset = 0) {
    return _buffers[NonUniformResourceIndex(handle)].Load<T>(byte_offset);
  }

  public T load_indexed(uint index) {
    return _buffers[NonUniformResourceIndex(handle)].Load<T>(index * sizeof(T));
  }

  public __subscript(uint index)->T {
    get { return load_indexed(index); }
  // set {}
  }
};

public struct ReadWriteBuffer<T> {
  uint handle;

  public T load(uint byte_offset = 0) {
    return _rw_buffers[NonUniformResourceIndex(handle)].Load<T>(byte_offset);
  }

  public T load_indexed(uint index) {
    return _rw_buffers[NonUniformResourceIndex(handle)].Load<T>(index *
                                                                sizeof(T));
  }

  public void store(T value, uint byte_offset = 0) {
    _rw_buffers[NonUniformResourceIndex(handle)].Store<T>(byte_offset, value);
  }

  public void store_indexed(T value, uint index) {
    _rw_buffers[NonUniformResourceIndex(handle)].Store<T>(index * sizeof(T),
                                                          value);
  }

  public __subscript(uint index)->T {
    get { return load_indexed(index); }
  // an issue?
    set { store_indexed(newValue, index); }
  }

  public uint interlocked_add(uint byte_offset, uint value) {
    uint old;
    _rw_buffers[NonUniformResourceIndex(handle)].InterlockedAdd(byte_offset,
                                                                value, old);
    return old;
  }

  public uint interlocked_min(uint byte_offset, uint value) {
    uint old;
    _rw_buffers[NonUniformResourceIndex(handle)].InterlockedMin(byte_offset,
                                                                value, old);
    return old;
  }

  public uint interlocked_max(uint byte_offset, uint value) {
    uint old;
    _rw_buffers[NonUniformResourceIndex(handle)].InterlockedMax(byte_offset,
                                                                value, old);
    return old;
  }

  public uint interlocked_compare_exchange(uint byte_offset, uint compare,
                                           uint value) {
    uint old;
    _rw_buffers[NonUniformResourceIndex(handle)].InterlockedCompareExchange(
        byte_offset, compare, value, old);
    return old;
  }
};

public ReadOnlyBuffer<T> get_buffer<T>(uint handle) {
  ReadOnlyBuffer<T> buf;
  buf.handle = handle;
  return buf;
}

public ReadWriteBuffer<T> get_rw_buffer<T>(uint handle) {
  ReadWriteBuffer<T> buf;
  buf.handle = handle;
  return buf;
}
