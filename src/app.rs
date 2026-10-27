use sgpu::*;

use crate::camera::Camera;
use simple_window::Window;

pub struct Application {
    camera: Camera,
}

impl simple_window::Application for Application {
    fn new(window: &Window) -> Self {
        let size = window.inner_size();

        sgpu::sgpu_init(&SgpuInititizationInfo::default_from_window(window));

        Application {
            camera: Camera::new(glam::vec3(0.0, 32.0, 0.0), size.width as f32 / size.height as f32),
        }
    }

    fn update(&mut self, update_info: &simple_window::UpdateInfo) {
        self.camera.process_input(&update_info.input_manager, update_info.dt);
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        sgpu::wait_idle();
    }
}
