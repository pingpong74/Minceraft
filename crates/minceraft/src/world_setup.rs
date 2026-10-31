use std::time::Instant;

use renderer::{ChunkAllocation, ChunkAllocationInfo, Renderer};
use world::{
    chunk::Chunk,
    generate::generate_chunk,
    mesher::{Border, Borders, mesh_chunk},
};

const CHUNKS_X: std::ops::Range<i32> = -10..10;
const CHUNKS_Z: std::ops::Range<i32> = -10..10;
const CHUNKS_Y: std::ops::Range<i32> = 0..10;

use std::collections::HashMap;

pub fn build_static_world(renderer: &mut Renderer) -> Vec<ChunkAllocation> {
    let mut chunks: HashMap<(i32, i32, i32), Chunk> = HashMap::new();
    for cy in CHUNKS_Y {
        for cz in CHUNKS_Z {
            for cx in CHUNKS_X {
                chunks.insert((cx, cy, cz), generate_chunk(cx, cy, cz));
            }
        }
    }

    let mut allocations = Vec::new();
    let mut total_quads = 0u64;
    let mut failed = 0usize;

    let start = Instant::now();

    for cy in CHUNKS_Y {
        for cz in CHUNKS_Z {
            for cx in CHUNKS_X {
                let chunk = &chunks[&(cx, cy, cz)];

                let mesh = mesh_chunk(
                    chunk,
                    &Borders([
                        chunks.get(&(cx + 1, cy, cz)).map_or(Border::AIR, |chunk| Border::from_chunk(chunk, 0)),
                        chunks.get(&(cx - 1, cy, cz)).map_or(Border::AIR, |chunk| Border::from_chunk(chunk, 1)),
                        chunks.get(&(cx, cy + 1, cz)).map_or(Border::AIR, |chunk| Border::from_chunk(chunk, 2)),
                        chunks.get(&(cx, cy - 1, cz)).map_or(Border::AIR, |chunk| Border::from_chunk(chunk, 3)),
                        chunks.get(&(cx, cy, cz + 1)).map_or(Border::AIR, |chunk| Border::from_chunk(chunk, 4)),
                        chunks.get(&(cx, cy, cz - 1)).map_or(Border::AIR, |chunk| Border::from_chunk(chunk, 5)),
                    ]),
                );

                if mesh.quads.is_empty() {
                    continue;
                }

                let quad_count = mesh.quads.len() as u64;

                let allocation = renderer.allocate_chunk(ChunkAllocationInfo {
                    faces: mesh.quads,
                    origin: chunk.origin().extend(1.0).to_array(),
                    aabb_max: chunk.aabb_max().extend(1.0).to_array(),
                    aabb_min: chunk.origin().extend(1.0).to_array(),
                    faces_ranges: mesh.face_ranges,
                });

                match allocation {
                    Some(allocation) => {
                        total_quads += quad_count;
                        allocations.push(allocation);
                    }
                    None => failed += 1,
                }
            }
        }
    }

    println!("Static world: {} chunks, {} quads ({} failed), meshed in {:?}", allocations.len(), total_quads, failed, start.elapsed());

    allocations
}
