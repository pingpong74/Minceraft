use offset_allocator::{Allocation, Allocator};
use sgpu::{Buffer, BufferDescription, BufferUsage, MemoryType, buffer_as_slice_mut, create_buffer};
use shaders::{ChunkMeta, Face};

const QUAD_SIZE: u64 = size_of::<Face>() as u64; // 8 bytes

pub struct QuadAllocation {
    allocation: Allocation,
    pub num_of_quads: u32,
}

impl QuadAllocation {
    pub fn first_quad(&self) -> u32 {
        self.allocation.offset
    }

    pub fn byte_offset(&self) -> u64 {
        self.allocation.offset as u64 * QUAD_SIZE
    }
}

pub struct QuadBuffer {
    buffer: Buffer,
    allocator: Allocator,
}

impl QuadBuffer {
    pub fn new(num_of_quads: u32) -> Self {
        Self {
            buffer: create_buffer(&BufferDescription {
                size: num_of_quads as u64 * QUAD_SIZE,
                memory_type: MemoryType::DeviceLocal,
                usage: BufferUsage::STORAGE | BufferUsage::TRANSFER_DST,
            }),
            allocator: Allocator::new(num_of_quads),
        }
    }

    pub fn buffer(&self) -> Buffer {
        self.buffer
    }

    pub fn allocate(&mut self, num_of_quads: u32) -> Option<QuadAllocation> {
        if num_of_quads == 0 {
            return None;
        }
        self.allocator.allocate(num_of_quads).map(|allocation| QuadAllocation { allocation, num_of_quads })
    }

    pub fn free(&mut self, allocation: QuadAllocation) {
        self.allocator.free(allocation.allocation);
    }
}

pub struct ChunkMetaBuffer {
    buffer: Buffer,
    free_indices: Vec<u32>,
    stride: u32,
    num_of_chunks: u32,
}

impl ChunkMetaBuffer {
    pub fn new(num_of_chunks: u32) -> Self {
        let stride = size_of::<ChunkMeta>() as u32;

        Self {
            buffer: create_buffer(&BufferDescription {
                usage: BufferUsage::STORAGE | BufferUsage::TRANSFER_DST,
                size: num_of_chunks as u64 * stride as u64,
                memory_type: MemoryType::DeviceLocal,
            }),
            free_indices: (0..num_of_chunks).rev().collect(),
            stride,
            num_of_chunks,
        }
    }

    pub fn buffer(&self) -> Buffer {
        self.buffer
    }

    pub fn size(&self) -> u64 {
        self.num_of_chunks as u64 * self.stride as u64
    }

    /// Returns a byte offset into the buffer.
    pub fn allocate(&mut self) -> Option<u32> {
        self.free_indices.pop().map(|index| index * self.stride)
    }

    /// Frees a previously allocated byte offset.
    pub fn free(&mut self, byte_offset: u32) {
        debug_assert_eq!(byte_offset % self.stride, 0);
        self.free_indices.push(byte_offset / self.stride);
    }
}

/// Bump allocator over host-visible memory, one region per frame slot.
/// slots = frames in flight
/// Call `begin_frame(slot)` after `acquire_image()` to clear it.
pub struct StagingBuffer {
    buffer: Buffer,
    slot_size: u64,
    base: u64,
    head: u64,
}

impl StagingBuffer {
    pub fn new(slot_size: u64, slots: usize) -> Self {
        Self {
            buffer: create_buffer(&BufferDescription {
                usage: BufferUsage::TRANSFER_SRC,
                size: slot_size * slots as u64,
                memory_type: MemoryType::PreferHost,
            }),
            slot_size,
            base: 0,
            head: 0,
        }
    }

    pub fn buffer(&self) -> Buffer {
        self.buffer
    }

    pub fn slot_size(&self) -> u64 {
        self.slot_size
    }

    pub fn remaining(&self) -> u64 {
        self.slot_size - self.head
    }

    pub fn begin_frame(&mut self, slot: usize) {
        self.base = slot as u64 * self.slot_size;
        self.head = 0;
    }

    pub fn copy<T: Copy>(&mut self, data: &[T]) -> Option<u64> {
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), size_of_val(data)) };

        if bytes.len() as u64 > self.remaining() {
            return None;
        }

        let offset = self.base + self.head;
        buffer_as_slice_mut::<u8>(&self.buffer)[offset as usize..offset as usize + bytes.len()].copy_from_slice(bytes);
        self.head += bytes.len() as u64;
        Some(offset)
    }
}
