use sgpu::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IndirectDrawCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub world_pos: [i32; 3],
    pub _pad: f32,
}

impl IndirectDrawCommand {
    pub fn zeroed() -> Self {
        IndirectDrawCommand {
            vertex_count: 0,
            instance_count: 0,
            first_vertex: 0,
            first_instance: 0,
            world_pos: [0; 3],
            _pad: 0.0,
        }
    }
}

pub struct IndirectDrawBuffer {
    buffer: Buffer,
    max_commands: usize,
    count: usize,
    free_indices: Vec<usize>,
}

impl IndirectDrawBuffer {
    pub fn new(max_commands: usize) -> Self {
        let capacity = max_commands * std::mem::size_of::<IndirectDrawCommand>();
        IndirectDrawBuffer {
            buffer: create_buffer(&BufferDescription {
                size: capacity as u64,
                usage: BufferUsage::STORAGE | BufferUsage::INDIRECT | BufferUsage::TRANSFER_DST,
                memory_type: MemoryType::DeviceLocal,
            }),
            max_commands,
            count: 0,
            free_indices: Vec::new(),
        }
    }

    pub fn allocate_slot(&mut self) -> usize {
        if let Some(idx) = self.free_indices.pop() {
            return idx;
        }
        let idx = self.count;
        self.count += 1;
        if idx >= self.max_commands {
            panic!("IndirectDrawBuffer: out of pre-allocated slots");
        }
        idx
    }

    pub fn free_slot(&mut self, slot: usize) {
        self.free_indices.push(slot);
    }

    pub fn slot_offset(&self, slot: usize) -> u64 {
        (slot * std::mem::size_of::<IndirectDrawCommand>()) as u64
    }

    pub fn raw(&self) -> Buffer {
        self.buffer
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn stride(&self) -> u32 {
        std::mem::size_of::<IndirectDrawCommand>() as u32
    }
}

impl Drop for IndirectDrawBuffer {
    fn drop(&mut self) {
        destroy_buffer(self.buffer);
    }
}
