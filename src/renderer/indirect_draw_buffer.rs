use sgpu::*;

use super::{BufferLocation, StagingBuffer};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IndirectDrawCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32, // what does this do??
    pub first_instance: u32,
    pub world_pos: [i32; 3],
    pub _pad: f32,
}

pub struct IndirectDrawBuffer {
    buffer: Buffer,
    staging: StagingBuffer,
    staging_offset: usize,
    max_commands: usize,
    count: usize,
    free_indices: Vec<usize>,
    pending_uploads: Vec<(u64, u64, u64)>,
}

impl IndirectDrawBuffer {
    pub fn new(max_commands: usize) -> Self {
        let command_size = std::mem::size_of::<IndirectDrawCommand>();
        let capacity = max_commands * command_size;

        return IndirectDrawBuffer {
            buffer: create_buffer(&BufferDescription {
                size: capacity as u64,
                usage: BufferUsage::STORAGE | BufferUsage::INDIRECT | BufferUsage::TRANSFER_DST,
                memory_type: MemoryType::DeviceLocal,
            }),
            staging: StagingBuffer::new(1 << 20),
            staging_offset: 0,
            max_commands,
            count: 0,
            free_indices: Vec::new(),
            pending_uploads: Vec::new(),
        };
    }

    pub fn allocate_command(&mut self, cmd: &IndirectDrawCommand) -> BufferLocation {
        let idx = self.free_indices.pop().unwrap_or_else(|| {
            let idx = self.count;
            self.count += 1;
            idx
        });

        if idx >= self.max_commands {
            self.grow();
            return self.allocate_command(cmd);
        }

        let command_size = std::mem::size_of::<IndirectDrawCommand>();
        let device_offset = (idx * command_size) as u64;

        if self.staging_offset + command_size > self.staging.capacity() {
            self.grow_staging(command_size);
        }

        self.staging.write_at(std::slice::from_ref(cmd), self.staging_offset as u64);
        self.pending_uploads.push((self.staging_offset as u64, device_offset, command_size as u64));
        self.staging_offset += command_size;

        return BufferLocation {
            offset: device_offset,
            size: command_size as u64,
        };
    }

    fn grow(&mut self) {
        let command_size = std::mem::size_of::<IndirectDrawCommand>();
        let new_max_commands = self.max_commands * 2;
        let new_capacity = new_max_commands * command_size;

        let new_buffer = create_buffer(&BufferDescription {
            size: new_capacity as u64,
            usage: BufferUsage::STORAGE | BufferUsage::INDIRECT | BufferUsage::TRANSFER_DST,
            memory_type: MemoryType::DeviceLocal,
        });

        destroy_buffer(self.buffer);
        self.buffer = new_buffer;
        self.max_commands = new_max_commands;
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

    pub fn free_command(&mut self, loc: BufferLocation) {
        let idx = (loc.offset / std::mem::size_of::<IndirectDrawCommand>() as u64) as usize;
        self.free_indices.push(idx);
    }

    pub fn raw(&self) -> Buffer {
        self.buffer
    }

    pub fn staging(&self) -> &StagingBuffer {
        &self.staging
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn stride(&self) -> u32 {
        std::mem::size_of::<IndirectDrawCommand>() as u32
    }

    pub fn flush_uploads(&mut self, cmd: &mut CommandBuffer) {
        for upload in self.pending_uploads.drain(0..) {
            cmd.copy_buffer(self.staging.raw(), self.buffer, upload.0, upload.1, upload.2);
        }
        self.staging_offset = 0;
    }
}
