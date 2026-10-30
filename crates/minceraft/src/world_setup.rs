use std::time::Instant;

use renderer::{ChunkAllocation, Renderer};
use world::{
    chunk::Chunk,
    generate::generate_chunk,
    mesher::{Neighbours, mesh_chunk},
};

const CHUNKS_X: std::ops::Range<i32> = -10..10;
const CHUNKS_Z: std::ops::Range<i32> = -10..10;
const CHUNKS_Y: std::ops::Range<i32> = 0..10;

use std::collections::HashMap;

pub fn build_static_world(renderer: &mut Renderer) -> Vec<ChunkAllocation> {
    let start = Instant::now();

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

    for cy in CHUNKS_Y {
        for cz in CHUNKS_Z {
            for cx in CHUNKS_X {
                let chunk = &chunks[&(cx, cy, cz)];

                let neighbours = Neighbours {
                    xp: chunks.get(&(cx + 1, cy, cz)),
                    xn: chunks.get(&(cx - 1, cy, cz)),
                    yp: chunks.get(&(cx, cy + 1, cz)),
                    yn: chunks.get(&(cx, cy - 1, cz)),
                    zp: chunks.get(&(cx, cy, cz + 1)),
                    zn: chunks.get(&(cx, cy, cz - 1)),
                };

                let mesh = mesh_chunk(chunk, &neighbours);

                if mesh.quads.is_empty() {
                    continue;
                }

                let base_meta = chunk.meta(mesh.face_ranges);
                let quad_count = mesh.quads.len() as u64;

                let allocation = renderer.allocate_chunk(mesh.quads, |alloc| {
                    let mut meta = base_meta;
                    for range in &mut meta.face_ranges {
                        range.first_quad += alloc.first_quad();
                    }
                    meta
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

    println!("Static world: {} chunks, {} quads ({} failed), built in {:?}", allocations.len(), total_quads, failed, start.elapsed());

    allocations
}
