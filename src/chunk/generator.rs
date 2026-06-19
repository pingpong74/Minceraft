use super::*;
use noise::*;

pub struct Generator {
    simplex: Simplex,
}

impl Generator {
    pub fn new(seed: u32) -> Generator {
        return Generator { simplex: Simplex::new(seed) };
    }

    pub fn sample_height(&self, x: i32, z: i32) -> u32 {
        let height = self.simplex.get([x as f64 * 0.01, z as f64 * 0.01]);

        //return 10;
        return (height * 30.0).max(0.0) as u32;
    }

    pub fn generate_blocks(&self, x: i32, y: i32, z: i32) -> [Block; CHUNK_VOLUME] {
        let mut blocks = [Block::AIR; CHUNK_VOLUME];

        for cx in 0..CHUNK_SIDE {
            for cz in 0..CHUNK_SIDE {
                let wx = x + cx as i32;
                let wz = z + cz as i32;
                let height = self.sample_height(wx, wz);

                for cy in 0..CHUNK_SIDE {
                    let wy = y + cy as i32;
                    if (wy as u32) < height {
                        blocks[Chunk::get_index(cx, cy, cz)] = Block::GRASS;
                    }
                }
            }
        }

        return blocks;
    }
}
