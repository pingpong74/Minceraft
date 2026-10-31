#![no_std]
use sgpu_shaders::{DrawIndirectCommand, glam::Vec3};

pub mod cull;
pub mod mesh;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceRange {
    pub first_quad: u32,
    pub quad_count: u32,
}

// aabb min and origin are repeated, later on remove them
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChunkMeta {
    pub origin: [f32; 4],
    pub aabb_min: [f32; 4],
    pub aabb_max: [f32; 4],
    pub face_ranges: [FaceRange; 6],
}

impl ChunkMeta {
    pub const IGNORE: ChunkMeta = ChunkMeta {
        origin: [0.0; 4],
        aabb_max: [0.0; 4],
        aabb_min: [0.0; 4],
        face_ranges: [FaceRange { first_quad: 0, quad_count: 0 }; 6],
    };

    // find a better way to do this, pointless comps
    pub fn should_ignore(&self) -> bool {
        *self == Self::IGNORE
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ChunkIndirectDrawCommand {
    pub draw_command: DrawIndirectCommand,
    pub world_position: [f32; 4],
}

#[derive(Clone, Copy)]
pub struct UnpackedFace {
    pub rel_pos: Vec3,
    pub width: u32,
    pub height: u32,
    pub face: u32,
    pub block_id: u32,
    pub tint: u32,
    pub ao: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Face(u32, u32);

impl Face {
    pub fn pack(x: u32, y: u32, z: u32, width: u32, height: u32, face: u32, block_id: u32, tint: u32, ao: u32, flags: u32) -> Self {
        let width = width.clamp(1, 32) - 1;
        let height = height.clamp(1, 32) - 1;

        let data = ((x as u64) & 0x1F)
            | (((y as u64) & 0x1F) << 5)
            | (((z as u64) & 0x1F) << 10)
            | (((width as u64) & 0x1F) << 15)
            | (((height as u64) & 0x1F) << 20)
            | (((face as u64) & 0x7) << 25)
            | (((block_id as u64) & 0xFFFF) << 28)
            | (((tint as u64) & 0xFF) << 44)
            | (((ao as u64) & 0xFF) << 52)
            | (((flags as u64) & 0xF) << 60);

        Self(data as u32, (data >> 32) as u32)
    }

    pub fn unpack(self) -> UnpackedFace {
        UnpackedFace {
            rel_pos: Vec3::new((self.0 & 0x1F) as f32, ((self.0 >> 5) & 0x1F) as f32, ((self.0 >> 10) & 0x1F) as f32),
            width: ((self.0 >> 15) & 0x1F) + 1,
            height: ((self.0 >> 20) & 0x1F) + 1,
            face: (self.0 >> 25) & 0x7,
            block_id: (self.0 >> 28) | ((self.1 & 0xFFF) << 4),
            tint: (self.1 >> 12) & 0xFF,
            ao: (self.1 >> 20) & 0xFF,
            flags: (self.1 >> 28) & 0xF,
        }
    }
}
