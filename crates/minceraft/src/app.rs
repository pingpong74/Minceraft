use crate::camera::{Camera, frustum_from_view_proj};
use crate::world_setup::build_static_world;
use renderer::{ChunkAllocation, FrameInput, Renderer};
use simple_window::Window;

pub struct Application {
    camera: Camera,
    renderer: Renderer,
    /// Kept alive so the static world stays visible; never freed.
    _chunks: Vec<ChunkAllocation>,
}

impl simple_window::Application for Application {
    fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let camera = Camera::new(glam::vec3(0.0, 180.0, 200.0), size.width as f32 / size.height as f32, -std::f64::consts::FRAC_PI_2, -0.35);
        let mut renderer = Renderer::new(window);
        let _chunks = build_static_world(&mut renderer);

        Application { camera, renderer, _chunks }
    }

    fn update(&mut self, update_info: &simple_window::UpdateInfo) {
        self.camera.process_input(&update_info.input_manager, update_info.dt);
        let cam_pos = self.camera.position();
        let view_proj = self.camera.view_proj();

        self.renderer.render(&FrameInput {
            width: update_info.width,
            height: update_info.height,
            view_proj: view_proj.to_cols_array(),
            camera_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 1.0],
            frustum_planes: frustum_from_view_proj(view_proj),
        });
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.camera.resize(width, height);
        self.renderer.resize(width, height);
    }
}
