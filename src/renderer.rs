use vulkano::{
    VulkanLibrary,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    format::Format,
    image::{Image, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{self, StandardMemoryAllocator},
    pipeline::{
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipeline, GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::{PipelineDescriptorSetLayoutCreateInfo, PipelineLayout},
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};

use winit::{event_loop::EventLoop, window::Window};

use super::mesh::BlockVertex;

use std::sync::Arc;

//Shaders
mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 460

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
    }
}

struct FrameData {
    cmd_builder: Arc<
        AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>,
            Arc<StandardCommandBufferAllocator>,
        >,
    >,
    framebuffers: Arc<Framebuffer>,
}

impl FrameData {
    fn new(
        image: Arc<Image>,
        render_pass: Arc<RenderPass>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        queue_family_index: u32,
    ) -> Self {
        let view = ImageView::new_default(image.clone()).unwrap();
        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },
        )
        .unwrap();
        let command_buffer_builder = Arc::new(
            AutoCommandBufferBuilder::primary(
                &command_buffer_allocator,
                queue_family_index,
                CommandBufferUsage::OneTimeSubmit,
            )
            .expect("Oojwgnowg"),
        );

        return FrameData {
            cmd_builder: command_buffer_builder,
            framebuffers: framebuffer,
        };
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct VulkanContext {
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    compute_queue: Arc<Queue>,
    transfer_queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    cmd_allocator: Arc<StandardCommandBufferAllocator>,
    framebuffers: Arc<FrameData>,
}

#[allow(unused)]
impl VulkanContext {
    pub fn new(window: Arc<Window>, event_loop: &EventLoop<()>) -> Self {
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let required_extensions = Surface::required_extensions(&event_loop);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create instance");

        //Store the dimmensions of the window, will be used later for swapchain creation
        let dimensions = window.inner_size();

        let surface =
            Surface::from_window(instance.clone(), window).expect("Failed to create surface");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, presetation_family) = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            //Only pick the required extensions and presentaion queue family
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            //Pick the discrete gpu
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .expect("no devices available");

        let mut graphics_family_index: Option<u32> = None;
        let mut compute_family_index: Option<u32> = None;
        let mut transfer_family_index: Option<u32> = None;

        // Iterate over all queue families
        for (index, queue_family) in physical_device.queue_family_properties().iter().enumerate() {
            let index = index as u32;
            let flags = queue_family.queue_flags;

            // Find graphics queue
            if graphics_family_index.is_none() && flags.contains(QueueFlags::GRAPHICS) {
                graphics_family_index = Some(index);
            }

            // Find compute-only queue (not graphics)
            if compute_family_index.is_none()
                && flags.contains(QueueFlags::COMPUTE)
                && !flags.contains(QueueFlags::GRAPHICS)
            {
                compute_family_index = Some(index);
            }

            // Find transfer-only queue (not graphics or compute)
            if transfer_family_index.is_none()
                && flags.contains(QueueFlags::TRANSFER)
                && !flags.contains(QueueFlags::GRAPHICS)
                && !flags.contains(QueueFlags::COMPUTE)
            {
                transfer_family_index = Some(index);
            }
        }

        let graphics_family_index = graphics_family_index.expect("No graphics queue found");
        let compute_family_index = compute_family_index.unwrap_or(graphics_family_index);
        let transfer_family_index = transfer_family_index.unwrap_or(graphics_family_index);

        if graphics_family_index != presetation_family {
            panic!("Presentaion and graphics familes are different, so problem :(((((((((((");
        }

        //Create device
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![
                    QueueCreateInfo {
                        queue_family_index: graphics_family_index,
                        ..Default::default()
                    },
                    QueueCreateInfo {
                        queue_family_index: compute_family_index,
                        ..Default::default()
                    },
                    QueueCreateInfo {
                        queue_family_index: transfer_family_index,
                        ..Default::default()
                    },
                ],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create device");

        let graphics_queue = queues.next().expect("Missing graphics queue");
        let compute_queue = queues.next().expect("Missing compute queue");
        let transfer_queue = queues.next().expect("Missing transfer queue");

        //Create swapchain
        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = physical_device
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap();

        //Make the graphics pipeline
        let vs_module =
            vertex_shader::load(device.clone()).expect("Failed to create vertex shader module");
        let vs = vs_module.entry_point("main").unwrap();

        let fs_module =
            fragment_shader::load(device.clone()).expect("failed to crate fragment shader module");
        let fs = fs_module.entry_point("main").unwrap();

        let vertex_input_state = BlockVertex::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: dimensions.into(),
            depth_range: 0.0..=1.0,
        };

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        let graphics_pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap();

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        let fd = Arc::new(FrameData::new(
            images[0].clone(),
            render_pass.clone(),
            command_buffer_allocator.clone(),
            graphics_queue.queue_family_index(),
        ));

        return VulkanContext {
            instance: instance,
            surface: surface,
            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            compute_queue: compute_queue,
            transfer_queue: transfer_queue,
            swapchain: swapchain,
            swapchain_images: images,
            graphics_pipeline: graphics_pipeline,
            cmd_allocator: command_buffer_allocator,
            framebuffers: fd,
        };
    }

    pub fn create_memory_allocator(&self) -> StandardMemoryAllocator {
        return StandardMemoryAllocator::new_default(self.device.clone());
    }

    pub fn update_camera() {}
}
