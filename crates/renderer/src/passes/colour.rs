use std::mem::size_of;

use sgpu::*;
use sgpu_taskgraph::*;

use shaders::mesh::MeshPushConstants;

use crate::{FrameData, Res};

pub(super) struct ChunkPassConfig {
    pub(super) pipeline: RasterizationPipeline,
    pub(super) sampler: Sampler,
    pub(super) max_draws: u32,
}

pub(super) struct ChunkPass {
    swapchain: PassImage<access::ColorWrite>,
    depth: PassImage<access::DepthWrite>,
    face_buffer: PassBuffer<access::StorageRead>,
    chunk_draw_command_buffer: PassBuffer<access::IndirectRead>,
    count_buffer: PassBuffer<access::IndirectRead>,
    texture: PassImage<access::FragmentSampledRead>,
    pipeline: RasterizationPipeline,
    sampler: Sampler,
    max_draws: u32,
}

impl Node<Res, FrameData> for ChunkPass {
    const QUEUE: QueueType = QueueType::Graphics;
    type Config = ChunkPassConfig;

    fn declare(res: &Res, config: &Self::Config, write: &mut NodeWriter) -> Self {
        Self {
            swapchain: write.image(res.swapchain),
            depth: write.image(res.depth),
            face_buffer: write.buffer(res.face_buffer),
            chunk_draw_command_buffer: write.buffer(res.chunk_draw_command_buffer),
            count_buffer: write.buffer(res.draw_count_buffer),
            texture: write.image(res.textures),
            pipeline: config.pipeline,
            sampler: config.sampler,
            max_draws: config.max_draws,
        }
    }

    fn execute(&self, data: &FrameData, context: &mut ExecutionContext<'_>) {
        let swapchain = context.image(self.swapchain).default_view();
        let depth = context.image(self.depth).default_view();

        let face_buffer = context.descriptor_index(self.face_buffer);
        let chunk_draw_command_buffer = context.buffer(self.chunk_draw_command_buffer);
        let counts = context.buffer(self.count_buffer);

        let texture = context.sampled_id(self.texture);
        let sampler = self.sampler.descriptor_index();

        context.cmd().begin_rendering(
            &RenderingBeginInfo {
                render_area: RenderArea {
                    offset: Offset2D { x: 0, y: 0 },
                    extent: data.extent,
                },
                color_attachments: &[RenderingAttachment {
                    image_view: swapchain,
                    clear_value: ClearValue::ColorFloat([0.45, 0.7, 0.95, 1.0]),
                    ..Default::default()
                }],
                depth_attachment: Some(RenderingAttachment {
                    image_view: depth,
                    clear_value: ClearValue::DepthStencil { depth: 1.0, stencil: 0 },
                    ..Default::default()
                }),
                ..Default::default()
            },
            |recorder| {
                recorder.bind_rasterization_pipeline(&self.pipeline);
                recorder.push_constants(&MeshPushConstants {
                    quad_buffer: face_buffer,
                    chunk_draw_cmd_buffer: chunk_draw_command_buffer.descriptor_index(),
                    texture,
                    sampler,
                    view_proj: data.view_proj,
                });
                recorder.set_viewport(data.extent.width, data.extent.height);
                recorder.set_scissor(data.extent.width, data.extent.height);
                recorder.draw_indirect_count(&chunk_draw_command_buffer, 0, &counts, 0, self.max_draws, size_of::<shaders::ChunkIndirectDrawCommand>() as u32);
            },
        );
    }
}
