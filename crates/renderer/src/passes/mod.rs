use sgpu::Sampler;
use sgpu_taskgraph::TaskGraph;

use crate::pipelines::Pipelines;
use crate::{FrameData, Res};

mod colour;
mod cull;
mod transfer;

pub(super) fn add_passes(task_graph: &mut TaskGraph<Res, FrameData>, pipelines: &Pipelines, sampler: Sampler, chunk_count: u32, max_draws: u32) {
    task_graph.add_node::<transfer::TransferPass>(());
    task_graph.add_node::<cull::ClearCounts>(());
    task_graph.add_node::<cull::CullMain>(cull::CullMainConfig {
        pipeline: pipelines.cull,
        chunk_count,
        max_draws,
    });
    task_graph.add_node::<colour::ChunkPass>(colour::ChunkPassConfig {
        pipeline: pipelines.colour,
        sampler,
        max_draws,
    });
}
