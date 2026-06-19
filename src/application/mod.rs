mod app;
mod input;

use std::time::{Duration, Instant};

pub use app::Application;
pub use input::InputManager;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::{Window, WindowAttributes},
};

pub struct Runner {
    app: Option<Application>,
    window: Option<Window>,

    last_frame: Instant,
    accumulator: Duration,
    tick_duration: Duration,

    dt: f64,
    alpha: f64,
}

impl Runner {
    pub fn new() -> Runner {
        return Runner {
            app: None,
            window: None,
            last_frame: Instant::now(),
            accumulator: Duration::new(0, 0),
            tick_duration: Duration::from_secs_f64(1.0 / 20.0),

            dt: 0.0,
            alpha: 0.0,
        };
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(WindowAttributes::default().with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))).expect("Failed to create window");

        window.set_cursor_grab(winit::window::CursorGrabMode::Locked).expect(":(");
        window.set_cursor_visible(false);

        self.app = Some(Application::new(&window));
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame);

        self.last_frame = now;
        self.accumulator += dt;

        let app = self.app.as_mut().unwrap();

        while self.accumulator >= self.tick_duration {
            app.fixed_update();
            self.accumulator -= self.tick_duration;
        }

        self.dt = dt.as_secs_f64();
        self.alpha = self.accumulator.as_secs_f64() / self.tick_duration.as_secs_f64();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
        if self.window.is_none() || self.app.is_none() {
            return;
        }

        let app = self.app.as_mut().unwrap();

        app.handle_window_event(&event);

        match event {
            WindowEvent::RedrawRequested => {
                app.update(self.dt);
            }
            WindowEvent::Resized(size) => {
                app.resize(size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &winit::event_loop::ActiveEventLoop, _: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        if let Some(app) = &mut self.app {
            app.handle_device_event(&event);
        }
    }
}
