use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct BlockVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}
