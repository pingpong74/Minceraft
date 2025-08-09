mod application;
mod mesh;
mod renderer;

use application::Application;
use renderer::VulkanContext;

use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new();

    let mut app = Application::new(&event_loop);
    app.run(event_loop);
}
