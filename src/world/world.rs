use crate::world::mesh::Chunk;
use cgmath::Vector3;
use std::collections::HashMap;

pub struct World {
    render_distance: u32,
    chunks: HashMap<Vector3<i32>, Chunk>,
}

impl World {
    fn new(render_distance: u32, player_pos: Vector3<f32>) -> Self {
        let chunks = HashMap::new();

        return World {
            render_distance: render_distance,
            chunks: chunks,
        };
    }

    fn update(&mut self, player_pos: Vector3<f32>) {}

    fn world_resize(&mut self, render_distance: u32) {}
}
