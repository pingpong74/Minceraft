use sgpu::*;

pub struct StagingBuffer {
    buffer: Buffer,
    capacity: usize,
}

impl StagingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: create_buffer(&BufferDescription {
                size: capacity as u64,
                usage: BufferUsage::TRANSFER_SRC,
                memory_type: MemoryType::PreferHost,
            }),
            capacity,
        }
    }

    pub fn write_at<T: Copy>(&self, data: &[T], byte_offset: u64) {
        let bytes = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * std::mem::size_of::<T>())
        };
        let dst = self.buffer.as_mut_slice::<u8>();
        dst[byte_offset as usize..byte_offset as usize + bytes.len()].copy_from_slice(bytes);
    }

    pub fn raw(&self) -> Buffer {
        self.buffer
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
