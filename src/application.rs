use super::camera::{Camera, CameraController};
use super::renderer::Renderer;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    window::Window,
};

const SPEED: f32 = 10.0;
const SENSTIVITY: f32 = 1.0;

pub enum GameState {
    Init,
    RenderFrame,
    Resize(u32, u32),
}

pub struct Application {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<GameState>>,
    renderer: Option<Renderer>,
    camera: Option<Camera>,
    camera_controller: CameraController,
    last_frame: std::time::Instant,
}

impl Application {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        return Self {
            renderer: None,
            camera: None,
            camera_controller: CameraController::new(SPEED, SENSTIVITY),
            last_frame: std::time::Instant::now(),
            #[cfg(target_arch = "wasm32")]
            proxy,
        };
    }
}

#[allow(unused)]
#[allow(non_snake_case)]
impl ApplicationHandler<GameState> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        let size = window.inner_size();

        let camera = Camera::new(size.width, size.height);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the
            self.renderer = Some(pollster::block_on(Renderer::new(window, &camera)).unwrap());
        }

        self.camera = Some(camera);

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(Init).is_ok())
                });
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let renderer = match &mut self.renderer {
            Some(c) => c,
            None => return,
        };

        let camera = match &mut self.camera {
            Some(c) => c,
            None => return,
        };

        self.camera_controller.process_event(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                camera.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                renderer.render(&camera);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera_controller
                .process_mouse_motion(delta.0, delta.1);
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: GameState) {}

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let delta_time = (std::time::Instant::now() - self.last_frame).as_secs_f32();
        self.last_frame = std::time::Instant::now();

        let camera = match &mut self.camera {
            Some(c) => c,
            None => return,
        };

        let f = camera.target;

        print!("\x1B[2J\x1B[1;1H");
        print!(
            "FPS: {:>6.2}    Pos: {:>6.2} {:>6.2} {:>6.2}    Looking Towards: {:>6.2} {:>6.2} {:>6.2}",
            (1.0 / delta_time),
            camera.eye.x,
            camera.eye.y,
            camera.eye.z,
            f.x,
            f.y,
            f.z
        );
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        self.camera_controller.update_camera(camera, delta_time);
    }
}
