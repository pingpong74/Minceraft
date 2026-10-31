mod buffers;
mod passes;
mod pipelines;
mod texture;
mod upload;

use sgpu::*;
use sgpu_taskgraph::{Id, TaskGraph};
use shaders::ChunkIndirectDrawCommand;
use simple_window::Window;

use crate::{passes::add_passes, pipelines::Pipelines, texture::TextureArray, upload::ChunkUploader};

pub use upload::{ChunkAllocation, ChunkAllocationInfo};

const CHUNK_COUNT: u32 = 10000;
const MAX_DRAWS: u32 = 10000;
const QUAD_COUNT: u32 = CHUNK_COUNT * 10000;
const STAGING_SLOT_SIZE: u64 = 8 << 20;
const MAX_FRAME_SLOTS: usize = 4;

struct Res {
    swapchain: Id<Image>,
    depth: Id<Image>,
    face_buffer: Id<Buffer>,
    chunk_data_buffer: Id<Buffer>,
    chunk_draw_command_buffer: Id<Buffer>,
    draw_count_buffer: Id<Buffer>,
    staging_buffer: Id<Buffer>,
    textures: Id<Image>,
}

/// Camera input from the application.
pub struct FrameInput {
    pub width: u32,
    pub height: u32,
    pub view_proj: [f32; 16],
    pub frustum_planes: [[f32; 4]; 6],
    pub camera_pos: [f32; 4],
}

struct FrameData {
    extent: Extent2D,
    view_proj: [f32; 16],
    frustum_planes: [[f32; 4]; 6],
    camera_pos: [f32; 4],
    quad_copies: Vec<BufferCopyRegion>,
    meta_copies: Vec<BufferCopyRegion>,
}

pub struct Renderer {
    graph: TaskGraph<'static, Res, FrameData>,
    swapchain: Swapchain,
    res: Res,
    uploader: ChunkUploader,

    _pipelines: Pipelines,
    _textures: TextureArray,
}

impl Renderer {
    pub fn new(window: &Window, debug: bool, texture_names: &'static [&'static str]) -> Self {
        let size = window.inner_size();

        sgpu::sgpu_init(&SgpuInititizationInfo::default_from_window(window));

        let swapchain = create_swapchain(
            window,
            &SwapchainDescription {
                format: Format::Rgba16Float,
                frames_in_flight: 3,
                width: size.width,
                height: size.height,
            },
        );
        assert!(swapchain.frame_count() <= MAX_FRAME_SLOTS);

        let mut graph = TaskGraph::<'static, Res, FrameData>::new();
        graph.set_frames_in_flight(swapchain.frame_count());

        let _pipelines = Pipelines::new();
        let _textures = TextureArray::new(texture_names);
        let uploader = ChunkUploader::new(CHUNK_COUNT, QUAD_COUNT, STAGING_SLOT_SIZE, MAX_FRAME_SLOTS);

        let res = Res {
            swapchain: graph.add_image(Image::default()),
            depth: graph.create_persistent_image(
                &ImageDescription {
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    format: Format::D32Float,
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
                },
                true,
            ),
            face_buffer: graph.add_buffer(uploader.quad_buffer()),
            chunk_data_buffer: graph.add_buffer(uploader.chunk_meta_buffer()),
            staging_buffer: graph.add_buffer(uploader.staging_buffer()),
            chunk_draw_command_buffer: graph.create_transient_buffer(&BufferDescription {
                usage: BufferUsage::STORAGE | BufferUsage::INDIRECT,
                size: MAX_DRAWS as u64 * size_of::<ChunkIndirectDrawCommand>() as u64,
                memory_type: MemoryType::DeviceLocal,
            }),
            draw_count_buffer: graph.create_transient_buffer(&BufferDescription {
                usage: BufferUsage::STORAGE | BufferUsage::INDIRECT | BufferUsage::TRANSFER_DST,
                size: size_of::<u32>() as u64,
                memory_type: MemoryType::DeviceLocal,
            }),
            textures: graph.add_image(_textures.texture()),
        };

        add_passes(&mut graph, &_pipelines, _textures.sampler(), CHUNK_COUNT, MAX_DRAWS);
        graph.present_target(res.swapchain);
        graph.compile(&res);

        graph.set_node_timing(debug);

        Self {
            graph,
            swapchain,
            res,
            uploader,
            _pipelines,
            _textures,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swapchain.resize(width, height);
        self.graph.resize(width, height);
    }

    pub fn render(&mut self, input: &FrameInput) {
        let acquired = self.swapchain.acquire_image();
        self.graph.set_image(self.res.swapchain, acquired.image());

        self.uploader.begin_frame(acquired.frame_index(), acquired.frames_in_flight());
        let (quad_copies, meta_copies) = self.uploader.take_copies();

        let data = FrameData {
            extent: Extent2D { width: input.width, height: input.height },
            view_proj: input.view_proj,
            frustum_planes: input.frustum_planes,
            camera_pos: input.camera_pos,
            quad_copies,
            meta_copies,
        };

        let counter = self.graph.execute_with_swapchain(&data, &acquired);
        self.swapchain.present(&acquired, counter);

        if let Some(timings) = self.graph.node_timings() {
            for node in timings.nodes {
                println!("{}: {:.3} ms", node.label, node.duration_ns / 1_000_000.0);
            }
            println!("\n\n\n");
        }
    }

    pub fn allocate_chunk(&mut self, alloc_info: ChunkAllocationInfo) -> Option<ChunkAllocation> {
        self.uploader.allocate_chunk(alloc_info)
    }

    pub fn free(&mut self, chunk_allocation: ChunkAllocation) {
        self.uploader.free_chunk(chunk_allocation);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        wait_idle();
    }
}
