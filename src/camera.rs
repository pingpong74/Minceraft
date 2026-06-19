use crate::application::InputManager;
use glam::{Mat4, Vec3};
use winit::{dpi::PhysicalSize, keyboard::KeyCode};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f64,
    pub pitch: f64,

    pub move_speed: f32,
    pub mouse_sensitivity: f64,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(position: Vec3, aspect: f32) -> Camera {
        return Camera {
            position,
            yaw: -0.769,
            pitch: -0.3,
            move_speed: 25.0,
            mouse_sensitivity: 0.002,
            aspect,
            fovy: 70_f32.to_radians(),
            znear: 0.1,
            zfar: 1000000.0,
        };
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
    }

    pub fn forward(&self) -> Vec3 {
        return Vec3::new((self.yaw.cos() * self.pitch.cos()) as f32, (self.pitch.sin()) as f32, (self.yaw.sin() * self.pitch.cos()) as f32).normalize();
    }

    pub fn right(&self) -> Vec3 {
        return self.forward().cross(Vec3::Y).normalize();
    }

    pub fn up(&self) -> Vec3 {
        return self.right().cross(self.forward()).normalize();
    }

    pub fn focal_lenght(&self) -> f32 {
        return 1.0 / (self.fovy * 0.5).tan();
    }

    pub fn projection(&self) -> Mat4 {
        let mut m = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        m.y_axis.y *= -1.0;
        return m;
    }

    pub fn view(&self) -> Mat4 {
        return Mat4::look_at_rh(self.position, self.position + self.forward(), self.up());
    }

    pub fn view_proj(&self) -> Mat4 {
        return self.projection() * self.view();
    }

    pub fn process_input(&mut self, input: &InputManager, dt: f64) {
        let (dx, dy) = input.mouse_delta();
        self.yaw -= dx * self.mouse_sensitivity;
        self.pitch -= dy * self.mouse_sensitivity;

        let max_pitch = 89_f64.to_radians();
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);

        if input.just_pressed(KeyCode::KeyE) {
            println!("Pos: {} Yaw: {} Pitch: {}", self.position, self.yaw, self.pitch);
        }

        let mut vel = Vec3::ZERO;

        if input.is_key_down(KeyCode::KeyW) {
            vel += self.forward();
        }
        if input.is_key_down(KeyCode::KeyS) {
            vel -= self.forward();
        }
        if input.is_key_down(KeyCode::KeyD) {
            vel += self.right();
        }
        if input.is_key_down(KeyCode::KeyA) {
            vel -= self.right();
        }
        if input.is_key_down(KeyCode::Space) {
            vel += self.up();
        }
        if input.is_key_down(KeyCode::ShiftLeft) {
            vel -= self.up();
        }

        if vel.length_squared() > 0.0 {
            vel = vel.normalize();
        }

        self.position += vel * self.move_speed * (dt as f32);
    }
}
