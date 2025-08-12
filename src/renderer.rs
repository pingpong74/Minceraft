use std::sync::Arc;

use wgpu::{hal::noop::Encoder, util::DeviceExt};
use winit::window::Window;

use super::mesh;

#[allow(unused)]
pub struct GpuContext {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    sufrace_config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<GpuContext> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::VULKAN,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        return Ok(GpuContext {
            instance: instance,
            adapter: adapter,
            surface: surface,
            sufrace_config: config,
            is_surface_configured: false,
            window: window,
            device: device,
            queue: queue,
        });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            println!("Resizing");
            self.sufrace_config.width = width;
            self.sufrace_config.height = height;
            self.surface.configure(&self.device, &self.sufrace_config);
            self.is_surface_configured = true;
        }
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn is_configured(&self) -> bool {
        return self.is_surface_configured;
    }

    fn get_surface_output(&self) -> wgpu::SurfaceTexture {
        return self
            .surface
            .get_current_texture()
            .expect("Failed to get surface texture");
    }

    fn create_encoder(&self) -> wgpu::CommandEncoder {
        return self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main encoder"),
            });
    }
}

pub struct RenderContext {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl RenderContext {
    pub async fn new(gpu_context: &GpuContext) -> anyhow::Result<RenderContext> {
        let shader = gpu_context
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

        let render_pipeline_layout =
            gpu_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            gpu_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[mesh::Vertex::get_vertex_descriptor()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gpu_context.sufrace_config.format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                        // or Features::POLYGON_MODE_POINT
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                    // Useful for optimizing shader compilation on Android
                    cache: None,
                });

        let vertex_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(mesh::VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        return Ok(RenderContext {
            pipeline: render_pipeline,
            vertex_buffer: vertex_buffer,
        });
    }
}

pub struct Renderer {
    gpu_context: GpuContext,
    render_context: RenderContext,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
        let gpu_context = GpuContext::new(window).await?;
        let rcx = RenderContext::new(&gpu_context).await?;

        // need to impl
        return Ok(Renderer {
            render_context: rcx,
            gpu_context: gpu_context,
        });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu_context.resize(width, height);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.gpu_context.request_redraw();

        if !self.gpu_context.is_configured() {
            return Ok(());
        }

        let output = self.gpu_context.get_surface_output();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu_context.create_encoder();

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            _render_pass.set_pipeline(&self.render_context.pipeline);
            _render_pass.set_vertex_buffer(0, self.render_context.vertex_buffer.slice(..));
            _render_pass.draw(0..(mesh::VERTICES.len() as u32), 0..1);
        }

        self.gpu_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        return Ok(());
    }
}
