use std::sync::Arc;

use wgpu::util::DeviceExt;
use wgpu::wgc::id::markers::BindGroup;
use winit::window::Window;

use super::camera::Camera;
use super::mesh;
use super::texture::Texture;

#[allow(unused)]
pub struct GpuContext {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    sufrace_config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<GpuContext> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
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
    depth_texture: Texture,
    pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
}

impl RenderContext {
    pub async fn new(gpu_context: &GpuContext, camera: &Camera) -> anyhow::Result<RenderContext> {
        let depth_texture = Texture::create_depth_texture(
            gpu_context,
            gpu_context.sufrace_config.width.max(1),
            gpu_context.sufrace_config.height.max(1),
        );

        let instance_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(mesh::FACES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let camera_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera buffer"),
                    contents: bytemuck::cast_slice(&camera.get_uniforms().view_proj),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let shader = gpu_context
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

        let camera_bind_group_layout =
            gpu_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let camera_bind_group = gpu_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        let render_pipeline_layout =
            gpu_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
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
                        buffers: &[mesh::FaceData::get_vertex_descriptor()],
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
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        return Ok(RenderContext {
            depth_texture: depth_texture,
            pipeline: render_pipeline,
            instance_buffer: instance_buffer,
            camera_buffer: camera_buffer,
            camera_bind_group: camera_bind_group,
        });
    }
}

pub struct Renderer {
    gpu_context: GpuContext,
    render_context: RenderContext,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, camera: &Camera) -> anyhow::Result<Renderer> {
        let gpu_context = GpuContext::new(window).await?;
        let rcx = RenderContext::new(&gpu_context, camera).await?;

        // need to impl
        return Ok(Renderer {
            render_context: rcx,
            gpu_context: gpu_context,
        });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.render_context.depth_texture =
            Texture::create_depth_texture(&self.gpu_context, width, height);
        self.gpu_context.resize(width, height);
    }

    pub fn render(&mut self, camera: &Camera) -> anyhow::Result<()> {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.render_context.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            _render_pass.set_pipeline(&self.render_context.pipeline);
            _render_pass.set_bind_group(0, &self.render_context.camera_bind_group, &[]);
            _render_pass.set_vertex_buffer(0, self.render_context.instance_buffer.slice(..));
            _render_pass.draw(0..6, 0..mesh::FACES.len() as u32);
        }

        self.gpu_context.queue.write_buffer(
            &self.render_context.camera_buffer,
            0,
            bytemuck::cast_slice(&camera.get_uniforms().view_proj),
        );
        self.gpu_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        return Ok(());
    }
}
