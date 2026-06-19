use crate::chunk::Face;
use sgpu::*;

use super::{BufferLocation, StagingBuffer};

#[derive(Clone, Copy)]
struct FreeBlock {
    offset: u64,
    size: u64,
}

pub struct FaceBuffer {
    buffer: Buffer,
    staging: StagingBuffer,
    staging_offset: usize,
    capacity: usize,
    free_blocks: Vec<FreeBlock>,
    pending_uploads: Vec<(u64, u64, u64)>,
}

impl FaceBuffer {
    pub fn new(capacity: usize) -> FaceBuffer {
        return FaceBuffer {
            buffer: create_buffer(&BufferDescription {
                size: capacity as u64,
                usage: BufferUsage::STORAGE | BufferUsage::TRANSFER_DST,
                memory_type: MemoryType::DeviceLocal,
            }),
            staging: StagingBuffer::new(1 << 20),
            staging_offset: 0,
            capacity: capacity,
            free_blocks: vec![FreeBlock { offset: 0, size: capacity as u64 }],
            pending_uploads: Vec::new(),
        };
    }

    pub fn allocate(&mut self, faces: &[Face]) -> BufferLocation {
        let device_size = (faces.len() * std::mem::size_of::<Face>()) as u64;
        let staging_size = device_size as usize;

        let device_offset = self.alloc_device(device_size);

        if self.staging_offset + staging_size > self.staging.capacity() {
            self.grow_staging(staging_size);
        }

        self.staging.write_at(faces, self.staging_offset as u64);
        self.pending_uploads.push((self.staging_offset as u64, device_offset, device_size));
        self.staging_offset += staging_size;

        return BufferLocation { offset: device_offset, size: device_size };
    }

    fn alloc_device(&mut self, size: u64) -> u64 {
        for i in 0..self.free_blocks.len() {
            if self.free_blocks[i].size >= size {
                let block = self.free_blocks[i];
                if block.size == size {
                    self.free_blocks.swap_remove(i);
                } else {
                    self.free_blocks[i].offset += size;
                    self.free_blocks[i].size -= size;
                }
                return block.offset;
            }
        }

        self.grow(size);
        self.alloc_device(size)
    }

    pub fn free(&mut self, loc: BufferLocation) {
        self.free_blocks.push(FreeBlock { offset: loc.offset, size: loc.size });
        self.coalesce();
    }

    pub fn flush_uploads(&mut self, cmd: &mut CommandBuffer) {
        for upload in self.pending_uploads.drain(0..) {
            cmd.copy_buffer(self.staging.raw(), self.buffer, upload.0, upload.1, upload.2);
        }
        self.staging_offset = 0;
    }

    pub fn raw(&self) -> Buffer {
        return self.buffer;
    }

    fn grow(&mut self, min_size: u64) {
        let needed = self.capacity + min_size as usize;
        let new_capacity = (self.capacity * 2).max(needed);

        let new_buffer = create_buffer(&BufferDescription {
            size: new_capacity as u64,
            usage: BufferUsage::STORAGE | BufferUsage::TRANSFER_DST,
            memory_type: MemoryType::DeviceLocal,
        });

        destroy_buffer(self.buffer);
        self.buffer = new_buffer;

        self.free_blocks.push(FreeBlock {
            offset: self.capacity as u64,
            size: (new_capacity - self.capacity) as u64,
        });
        self.capacity = new_capacity;

        self.coalesce();
    }

    fn grow_staging(&mut self, min_size: usize) {
        let needed = self.staging.capacity() + min_size;
        let new_capacity = (self.staging.capacity() * 2).max(needed);

        let new_staging = StagingBuffer::new(new_capacity);

        let old_raw = self.staging.raw();
        let src = old_raw.as_slice::<u8>();
        let new_raw = new_staging.raw();
        let dst = new_raw.as_mut_slice::<u8>();
        dst[..self.staging_offset].copy_from_slice(&src[..self.staging_offset]);

        self.staging = new_staging;
    }

    fn coalesce(&mut self) {
        self.free_blocks.sort_by_key(|b| b.offset);
        let mut i = 0;
        while i + 1 < self.free_blocks.len() {
            let end = self.free_blocks[i].offset + self.free_blocks[i].size;
            if end >= self.free_blocks[i + 1].offset {
                let next_end = self.free_blocks[i + 1].offset + self.free_blocks[i + 1].size;
                self.free_blocks[i].size = next_end - self.free_blocks[i].offset;
                self.free_blocks.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
}
