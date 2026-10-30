mod app;
mod camera;
mod world_setup;

use app::*;

fn main() {
    simple_window::run::<Application>(0.2);
}
