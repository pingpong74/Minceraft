use rand::random;

use crate::block::{AIR, FLOWER, MOSS, STONE};
use crate::chunk::{CHUNK_SIDE, Chunk, ChunkKey, Voxels};

pub fn generate_chunk(cx: i32, cy: i32, cz: i32) -> Chunk {
    let key = ChunkKey::new(cx, cy, cz, 0);

    let world_base_x = cx * CHUNK_SIDE as i32;
    let world_base_y = cy * CHUNK_SIDE as i32;
    let world_base_z = cz * CHUNK_SIDE as i32;

    let voxels = Voxels::from_fn(|lx, ly, lz| {
        let wx = (world_base_x + lx as i32) as f32;
        let wy = (world_base_y + ly as i32) as f32;
        let wz = (world_base_z + lz as i32) as f32;

        let height = terrain_height(wx, wz);

        let rand: f32 = random();

        if wy < height - 4.0 {
            STONE
        } else if wy <= height {
            if rand > 0.85 { FLOWER } else { MOSS }
        } else {
            AIR
        }
    });

    Chunk::new(key, voxels)
}

fn terrain_height(wx: f32, wz: f32) -> f32 {
    let h = 80.0 + (wx * 0.03).sin() * 25.0 + (wz * 0.025).cos() * 20.0 + ((wx + wz) * 0.015).sin() * 15.0 + ((wx * 0.08).sin() * (wz * 0.08).cos()) * 8.0;

    h.clamp(10.0, 280.0)
}
