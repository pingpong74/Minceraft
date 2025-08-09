use super::VulkanContext;
use std::sync::Arc;
use vulkano::memory::allocator::StandardMemoryAllocator;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Application {
    window: Arc<Window>,
    vulkan_context: Arc<VulkanContext>,
    memory_allocator: StandardMemoryAllocator,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
        let vulkan_context = Arc::new(VulkanContext::new(window.clone(), &event_loop));
        let memory_allocator = vulkan_context.create_memory_allocator();
        return Application {
            window: window,
            memory_allocator: memory_allocator,
            vulkan_context: vulkan_context,
        };
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        event_loop.run(|event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        });
    }
}
