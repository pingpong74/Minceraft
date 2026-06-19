use crate::chunk::Face;
use sgpu::*;

use super::BufferLocation;

#[derive(Clone, Copy)]
struct FreeBlock {
    offset: u64,
    size: u64,
}

pub struct FaceBuffer {
    buffer: Buffer,
    free_blocks: Vec<FreeBlock>,
}

impl FaceBuffer {
    pub fn new(max_faces: usize) -> FaceBuffer {
        let capacity = (max_faces * std::mem::size_of::<Face>()) as u64;
        FaceBuffer {
            buffer: create_buffer(&BufferDescription {
                size: capacity,
                usage: BufferUsage::STORAGE | BufferUsage::TRANSFER_DST,
                memory_type: MemoryType::DeviceLocal,
            }),
            free_blocks: vec![FreeBlock { offset: 0, size: capacity }],
        }
    }

    pub fn allocate(&mut self, num_faces: usize) -> BufferLocation {
        let size = (num_faces * std::mem::size_of::<Face>()) as u64;
        for i in 0..self.free_blocks.len() {
            if self.free_blocks[i].size >= size {
                let block = self.free_blocks[i];
                if block.size == size {
                    self.free_blocks.swap_remove(i);
                } else {
                    self.free_blocks[i].offset += size;
                    self.free_blocks[i].size -= size;
                }
                return BufferLocation { offset: block.offset, size };
            }
        }
        panic!("FaceBuffer: out of pre-allocated memory");
    }

    pub fn free(&mut self, loc: BufferLocation) {
        self.free_blocks.push(FreeBlock { offset: loc.offset, size: loc.size });
        self.coalesce();
    }

    pub fn raw(&self) -> Buffer {
        self.buffer
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

impl Drop for FaceBuffer {
    fn drop(&mut self) {
        destroy_buffer(self.buffer);
    }
}
