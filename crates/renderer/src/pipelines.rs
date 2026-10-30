use sgpu::*;

const SHADERS_SPV: &[u8] = include_bytes!("../../shaders/out/shaders.spv");

pub struct Pipelines {
    pub colour: RasterizationPipeline,
    pub cull: ComputePipeline,
}

impl Pipelines {
    pub fn new() -> Self {
        Self {
            colour: create_rasterization_pipeline(&RasterizationPipelineDescription {
                vertex: ShaderModule {
                    spirv: SHADERS_SPV,
                    entry: "mesh::mesh_vs",
                    ..Default::default()
                },
                fragment: ShaderModule {
                    spirv: SHADERS_SPV,
                    entry: "mesh::mesh_fs",
                    ..Default::default()
                },
                topology: PrimitiveTopology::TriangleList,
                cull_mode: CullMode::None,
                front_face: FrontFace::CounterClockwise,
                polygon_mode: PolygonMode::Fill,
                depth_stencil: DepthStencilState::default(),
                blend_mode: BlendMode::Opaque,
                outputs: PipelineOutputs {
                    color: &[Format::Rgba16Float],
                    depth: Some(Format::D32Float),
                    stencil: None,
                },
            }),
            cull: create_compute_pipeline(&ShaderModule {
                spirv: SHADERS_SPV,
                entry: "cull::cull_cs",
                ..Default::default()
            }),
        }
    }
}

impl Drop for Pipelines {
    fn drop(&mut self) {
        destroy_rasterization_pipeline(self.colour);
        destroy_compute_pipeline(self.cull);
    }
}
