mod app;
mod camera;
mod renderer;
mod world;

use app::*;

fn main() {
    simple_window::run::<Application>(0.2);
}
