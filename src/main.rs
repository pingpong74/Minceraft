mod application;
mod camera;
mod renderer;
mod texture;
mod world;

use application::Application;
use winit::event_loop::EventLoop;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Failed to create event loop");

    let mut app = Application::new();

    event_loop.run_app(&mut app);
}
