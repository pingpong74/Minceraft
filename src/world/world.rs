use crate::world::mesh;
use crate::world::mesh::Chunk;
use cgmath::{Point3, Vector3};
use std::collections::HashMap;

pub struct World {
    render_distance: u32,
    pub chunks: HashMap<Point3<i32>, Chunk>,
}

impl World {
    fn to_chunk_coords(pos: &Point3<f32>) -> Point3<i32> {
        return Point3::new(
            (pos.x / (mesh::CHUNK_SIZE as f32)).floor() as i32,
            (pos.y / (mesh::CHUNK_SIZE as f32)).floor() as i32,
            (pos.z / (mesh::CHUNK_SIZE as f32)).floor() as i32,
        );
    }

    pub fn new(render_distance: u32, player_pos: Point3<f32>) -> Self {
        let mut chunks = HashMap::new();

        let chunk_coords = World::to_chunk_coords(&player_pos);

        println!("{}  {}  {}", chunk_coords.x, chunk_coords.y, chunk_coords.z);

        let mut chunk = Chunk::new(chunk_coords);

        pollster::block_on(Chunk::generate(&mut chunk));
        pollster::block_on(Chunk::mesh(&mut chunk));

        chunks.insert(chunk_coords, Chunk::new(chunk_coords));

        return World {
            render_distance: render_distance,
            chunks: chunks,
        };
    }

    pub fn update(&mut self, player_pos: Point3<f32>) {}

    pub fn world_resize(&mut self, render_distance: u32) {}
}
