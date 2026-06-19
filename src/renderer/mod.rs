mod indirect_draw_buffer;
mod vertex_buffer;

pub use indirect_draw_buffer::{IndirectDrawBuffer, IndirectDrawCommand};
use sgpu::*;
pub use vertex_buffer::FaceBuffer;
use winit::dpi::PhysicalSize;

#[derive(Clone, Copy)]
pub struct BufferLocation {
    pub offset: u64,
    pub size: u64,
}

const VERTEX_SHADER: &[u8] = include_bytes!("../../shaders/compiled/vert.spv");
const FRAGMENT_SHADER: &[u8] = include_bytes!("../../shaders/compiled/frag.spv");

pub struct Renderer {
    pipeline: RasterizationPipeline,
    depth_image: Image,
    size: PhysicalSize<u32>,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct PushData {
    view_proj: [f32; 16],
    face_buffer_id: u32,
    indirecr_draw_buffer_id: u32,
}

impl Renderer {
    pub fn new(size: PhysicalSize<u32>) -> Renderer {
        let depth_image = create_image(&ImageDescription {
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            format: Format::D32Float,
            image_type: ImageType::Type2D,
            extent: Extent3D {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            memory_type: MemoryType::DeviceLocal,
            default_view: ImageViewDescription {
                subresources: ImageSubresources {
                    aspect: ImageAspect::DEPTH,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });

        let pipeline = create_rasterization_pipeline(&RasterizationPipelineDescription {
            vertex_shader: VERTEX_SHADER,
            fragment_shader: FRAGMENT_SHADER,
            topology: PrimitiveTopology::TriangleList,
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            polygon_mode: PolygonMode::Fill,
            depth_stencil: DepthStencilState {
                depth_test: true,
                depth_write: true,
                depth_compare: CompareOp::Less,
                stencil_test: false,
            },
            blend_mode: BlendMode::Opaque,
            outputs: PipelineOutputs {
                color: &[Format::Rgba16Float],
                depth: Some(Format::D32Float),
                stencil: None,
            },
        });

        return Renderer { pipeline, depth_image, size };
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        destroy_image(self.depth_image);
        self.depth_image = create_image(&ImageDescription {
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            format: Format::D32Float,
            image_type: ImageType::Type2D,
            extent: Extent3D {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            memory_type: MemoryType::DeviceLocal,
            default_view: ImageViewDescription {
                subresources: ImageSubresources {
                    aspect: ImageAspect::DEPTH,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });
        self.size = size;
    }

    pub fn render(&self, cmd: &mut CommandBuffer, swapchain_image: Image, face_buffer: &FaceBuffer, indirect_buffer: &IndirectDrawBuffer, view_proj: &glam::Mat4, chunk_count: u32) {
        cmd.image_barrier(&ImageBarrier {
            view: swapchain_image.default_view(),
            previous_accesses: &[AccessType::Present],
            next_accesses: &[AccessType::ColorAttachmentWrite],
            discard_contents: true,
            ..Default::default()
        });

        cmd.image_barrier(&ImageBarrier {
            view: self.depth_image.default_view(),
            previous_accesses: &[],
            next_accesses: &[AccessType::DepthStencilAttachmentWrite],
            discard_contents: true,
            ..Default::default()
        });

        cmd.begin_rendering(
            &RenderingBeginInfo {
                render_area: RenderArea {
                    offset: Offset2D { x: 0, y: 0 },
                    extent: Extent2D {
                        width: self.size.width,
                        height: self.size.height,
                    },
                },
                color_attachments: &[RenderingAttachment {
                    image_view: swapchain_image.default_view(),
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_value: ClearValue::ColorFloat([0.4, 0.6, 0.9, 1.0]),
                    ..Default::default()
                }],
                depth_attachment: Some(RenderingAttachment {
                    image_view: self.depth_image.default_view(),
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::DontCare,
                    clear_value: ClearValue::DepthStencil { depth: 1.0, stencil: 0 },
                    ..Default::default()
                }),
                ..Default::default()
            },
            |recorder| {
                recorder.set_viewport(self.size.width, self.size.height);
                recorder.set_scissor(self.size.width, self.size.height);
                recorder.bind_rasterization_pipeline(&self.pipeline);

                recorder.push_constants(&PushData {
                    view_proj: view_proj.to_cols_array(),
                    face_buffer_id: face_buffer.raw().descriptor_index(),
                    indirecr_draw_buffer_id: indirect_buffer.raw().descriptor_index(),
                });

                recorder.draw_indirect(&indirect_buffer.raw(), 0, chunk_count, indirect_buffer.stride());
            },
        );

        cmd.image_barrier(&ImageBarrier {
            view: swapchain_image.default_view(),
            previous_accesses: &[AccessType::ColorAttachmentWrite],
            next_accesses: &[AccessType::Present],
            ..Default::default()
        });
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        destroy_image(self.depth_image);
    }
}
