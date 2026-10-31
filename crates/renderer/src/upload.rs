use std::collections::VecDeque;

use sgpu::{BufferCopyRegion, QueueType, destroy_buffer, record, submit, wait};
use shaders::{ChunkMeta, Face, FaceRange};

use crate::buffers::{ChunkMetaBuffer, QuadAllocation, QuadBuffer, StagingBuffer};

const META_SIZE: u64 = size_of::<ChunkMeta>() as u64;

fn region(src_offset: u64, dst_offset: u64, size: u64) -> BufferCopyRegion {
    BufferCopyRegion { src_offset, dst_offset, size }
}

/// A chunk that lives on the GPU (or is queued to). Give it back with `free_chunk`.
pub struct ChunkAllocation {
    quad: QuadAllocation,
    meta_offset: u32,
}

pub struct ChunkAllocationInfo {
    pub faces: Vec<Face>,
    pub faces_ranges: [FaceRange; 6],
    pub origin: [f32; 4],
    pub aabb_min: [f32; 4],
    pub aabb_max: [f32; 4],
}

impl ChunkAllocation {
    pub fn first_quad(&self) -> u32 {
        self.quad.first_quad()
    }

    pub fn num_of_quads(&self) -> u32 {
        self.quad.num_of_quads
    }

    pub fn meta_offset(&self) -> u32 {
        self.meta_offset
    }
}

enum Pending {
    Show { faces: Vec<Face>, quad_offset: u64, meta: ChunkMeta, meta_offset: u32 },
    Hide { chunk: ChunkAllocation },
}

pub struct ChunkUploader {
    quad_buffer: QuadBuffer,
    chunk_meta_buffer: ChunkMetaBuffer,
    staging_buffer: StagingBuffer,

    /// FIFO of requests that have not been staged yet.
    pending: VecDeque<Pending>,
    /// Hidden chunks waiting until no in-flight frame can still draw them: (release_frame, chunk).
    retiring: Vec<(u64, ChunkAllocation)>,

    /// This frame's copies, consumed by `TransferPass` through `take_copies`.
    quad_copies: Vec<BufferCopyRegion>,
    meta_copies: Vec<BufferCopyRegion>,

    frame: u64,
}

impl ChunkUploader {
    pub fn new(num_of_chunks: u32, num_of_quads: u32, staging_slot_size: u64, staging_slots: usize) -> Self {
        let chunk_meta_buffer = ChunkMetaBuffer::new(num_of_chunks);

        debug_assert!(
            unsafe { std::slice::from_raw_parts((&ChunkMeta::IGNORE as *const ChunkMeta).cast::<u8>(), size_of::<ChunkMeta>()) }.iter().all(|b| *b == 0),
            "ChunkMeta::IGNORE must be all zero bytes for the fill below (or upload IGNORE instead)"
        );

        let mut recorder = record(QueueType::Transfer);
        recorder.fill_buffer(&chunk_meta_buffer.buffer(), 0, chunk_meta_buffer.size(), 0);
        wait(submit(&[recorder]));

        Self {
            quad_buffer: QuadBuffer::new(num_of_quads),
            chunk_meta_buffer,
            staging_buffer: StagingBuffer::new(staging_slot_size, staging_slots),
            pending: VecDeque::new(),
            retiring: Vec::new(),
            quad_copies: Vec::new(),
            meta_copies: Vec::new(),
            frame: 0,
        }
    }

    pub fn quad_buffer(&self) -> sgpu::Buffer {
        self.quad_buffer.buffer()
    }

    pub fn chunk_meta_buffer(&self) -> sgpu::Buffer {
        self.chunk_meta_buffer.buffer()
    }

    pub fn staging_buffer(&self) -> sgpu::Buffer {
        self.staging_buffer.buffer()
    }

    pub fn allocate_chunk(&mut self, chunk_alloc_info: ChunkAllocationInfo) -> Option<ChunkAllocation> {
        if chunk_alloc_info.faces.is_empty() || chunk_alloc_info.faces.len() as u64 * 8 + META_SIZE > self.staging_buffer.slot_size() {
            return None;
        }

        let quad = self.quad_buffer.allocate(chunk_alloc_info.faces.len() as u32)?;
        let Some(meta_offset) = self.chunk_meta_buffer.allocate() else {
            self.quad_buffer.free(quad);
            return None;
        };

        let chunk_allocation = ChunkAllocation { quad, meta_offset };

        let base_first_quad = chunk_allocation.first_quad();
        let mut face_ranges = chunk_alloc_info.faces_ranges;
        for range in &mut face_ranges {
            range.first_quad += base_first_quad;
        }

        let meta = ChunkMeta {
            origin: chunk_alloc_info.origin,
            aabb_min: chunk_alloc_info.aabb_min,
            aabb_max: chunk_alloc_info.aabb_max,
            face_ranges,
        };

        self.pending.push_back(Pending::Show {
            faces: chunk_alloc_info.faces,
            quad_offset: chunk_allocation.quad.byte_offset(),
            meta,
            meta_offset,
        });
        Some(chunk_allocation)
    }

    pub fn free_chunk(&mut self, chunk: ChunkAllocation) {
        self.pending.push_back(Pending::Hide { chunk });
    }

    /// Call once per frame right after `acquire_image()` to set the slot (frame in flight)
    pub fn begin_frame(&mut self, slot: usize, frames_in_flight: usize) {
        self.frame += 1;
        self.staging_buffer.begin_frame(slot);

        let frame = self.frame;
        let mut i = 0;
        while i < self.retiring.len() {
            if self.retiring[i].0 <= frame {
                let (_, chunk) = self.retiring.swap_remove(i);
                self.quad_buffer.free(chunk.quad);
                self.chunk_meta_buffer.free(chunk.meta_offset);
            } else {
                i += 1;
            }
        }

        // Stage as many requests as fit, in order. Whatever doesn't fit waits for the next frame.
        while let Some(op) = self.pending.front() {
            let fits = match op {
                Pending::Show { faces, .. } => faces.len() as u64 * 8 + META_SIZE <= self.staging_buffer.remaining(),
                Pending::Hide { chunk } => {
                    // Don't write the same meta slot twice in one copy call (order is not defined).
                    META_SIZE <= self.staging_buffer.remaining() && !self.meta_copies.iter().any(|r| r.dst_offset == chunk.meta_offset as u64)
                }
            };
            if !fits {
                break;
            }

            match self.pending.pop_front().unwrap() {
                Pending::Show { faces, quad_offset, meta, meta_offset } => {
                    let src = self.staging_buffer.copy(&faces).unwrap();
                    self.quad_copies.push(region(src, quad_offset, faces.len() as u64 * 8));

                    let src = self.staging_buffer.copy(&[meta]).unwrap();
                    self.meta_copies.push(region(src, meta_offset as u64, META_SIZE));
                }
                Pending::Hide { chunk } => {
                    let src = self.staging_buffer.copy(&[ChunkMeta::IGNORE]).unwrap();
                    self.meta_copies.push(region(src, chunk.meta_offset as u64, META_SIZE));
                    self.retiring.push((frame + frames_in_flight as u64, chunk));
                }
            }
        }
    }

    /// (quad copies, meta copies)
    pub fn take_copies(&mut self) -> (Vec<BufferCopyRegion>, Vec<BufferCopyRegion>) {
        (std::mem::take(&mut self.quad_copies), std::mem::take(&mut self.meta_copies))
    }
}

impl Drop for ChunkUploader {
    fn drop(&mut self) {
        destroy_buffer(self.quad_buffer.buffer());
        destroy_buffer(self.chunk_meta_buffer.buffer());
        destroy_buffer(self.staging_buffer.buffer());
    }
}
