mod application;
mod camera;
mod chunk;
mod renderer;

use application::*;
use winit::event_loop::EventLoop;

fn main() {
    let mut runner = Runner::new();
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    event_loop.run_app(&mut runner).expect("Application running failed");
}
