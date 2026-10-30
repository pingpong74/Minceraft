use sgpu::*;
use sgpu_taskgraph::*;

use shaders::cull::CullPushConstants;

use crate::{FrameData, Res};

pub(super) struct ClearCounts {
    counts: PassBuffer<access::TransferWrite>,
}

impl Node<Res, FrameData> for ClearCounts {
    const QUEUE: QueueType = QueueType::Compute;
    type Config = ();

    fn declare(res: &Res, _: &Self::Config, write: &mut NodeWriter) -> Self {
        Self { counts: write.buffer(res.draw_count_buffer) }
    }

    fn execute(&self, _data: &FrameData, context: &mut ExecutionContext<'_>) {
        let counts = context.buffer(self.counts);
        context.cmd().fill_buffer(&counts, 0, size_of::<u32>() as u64, 0);
    }
}

pub(super) struct CullMainConfig {
    pub(super) pipeline: ComputePipeline,
    pub(super) chunk_count: u32,
    pub(super) max_draws: u32,
}

pub(super) struct CullMain {
    chunk_meta: PassBuffer<access::ComputeRead>,
    draws: PassBuffer<access::ComputeWrite>,
    counts: PassBuffer<access::ComputeWrite>,
    pipeline: ComputePipeline,
    chunk_count: u32,
    max_draws: u32,
}

impl Node<Res, FrameData> for CullMain {
    const QUEUE: QueueType = QueueType::Compute;
    type Config = CullMainConfig;

    fn declare(res: &Res, config: &Self::Config, write: &mut NodeWriter) -> Self {
        Self {
            chunk_meta: write.buffer(res.chunk_data_buffer),
            draws: write.buffer(res.chunk_draw_command_buffer),
            counts: write.buffer(res.draw_count_buffer),
            pipeline: config.pipeline,
            chunk_count: config.chunk_count,
            max_draws: config.max_draws,
        }
    }

    fn execute(&self, data: &FrameData, context: &mut ExecutionContext<'_>) {
        let chunks = context.descriptor_index(self.chunk_meta);
        let draws = context.descriptor_index(self.draws);
        let counts = context.descriptor_index(self.counts);
        let cmd = context.cmd();

        cmd.bind_compute_pipeline(&self.pipeline);
        cmd.push_constants(&CullPushConstants {
            chunks,
            draws,
            counts,
            chunk_count: self.chunk_count,
            max_draws: self.max_draws,
            _pad: [0; 2],
            frustum_planes: data.frustum_planes,
            camera_pos: data.camera_pos,
        });
        cmd.dispatch(self.chunk_count.div_ceil(64), 1, 1);
    }
}
