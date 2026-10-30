use sgpu_taskgraph::{ExecutionContext, Node, NodeWriter, PassBuffer, access};

use crate::{FrameData, Res};

pub(super) struct TransferPass {
    staging: PassBuffer<access::TransferRead>,
    quads: PassBuffer<access::TransferWrite>,
    chunk_meta: PassBuffer<access::TransferWrite>,
}

impl Node<Res, FrameData> for TransferPass {
    type Config = ();
    const QUEUE: sgpu::QueueType = sgpu::QueueType::Transfer;

    fn declare(res: &Res, _config: &Self::Config, write: &mut NodeWriter) -> Self {
        Self {
            staging: write.buffer(res.staging_buffer),
            quads: write.buffer(res.face_buffer),
            chunk_meta: write.buffer(res.chunk_data_buffer),
        }
    }

    fn execute(&self, data: &FrameData, context: &mut ExecutionContext<'_>) {
        let staging = context.buffer(self.staging);
        let quads = context.buffer(self.quads);
        let chunk_meta = context.buffer(self.chunk_meta);
        let cmd = context.cmd();

        if !data.quad_copies.is_empty() {
            cmd.copy_buffer(staging, quads, &data.quad_copies);
        }
        if !data.meta_copies.is_empty() {
            cmd.copy_buffer(staging, chunk_meta, &data.meta_copies);
        }
    }
}
