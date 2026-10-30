use glam::{Mat4, Vec3};
use simple_window::{InputManager, KeyCode};

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
    pub fn new(position: Vec3, aspect: f32, yaw: f64, pitch: f64) -> Camera {
        Camera {
            position,
            yaw,
            pitch,
            move_speed: 25.0,
            mouse_sensitivity: 0.002,
            aspect,
            fovy: 70_f32.to_radians(),
            znear: 0.1,
            zfar: 1000000.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new((self.yaw.cos() * self.pitch.cos()) as f32, (self.pitch.sin()) as f32, (self.yaw.sin() * self.pitch.cos()) as f32).normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn projection(&self) -> Mat4 {
        glam::camera::rh::proj::vulkan::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }

    pub fn view(&self) -> Mat4 {
        glam::camera::rh::view::look_at_mat4(self.position, self.position + self.forward(), self.up())
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection() * self.view()
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

pub fn frustum_from_view_proj(view_proj: Mat4) -> [[f32; 4]; 6] {
    let row0 = view_proj.row(0);
    let row1 = view_proj.row(1);
    let row2 = view_proj.row(2);
    let row3 = view_proj.row(3);

    let raw_planes = [row3 + row0, row3 - row0, row3 + row1, row3 - row1, row2, row3 - row2];
    let mut planes = [[0.0; 4]; 6];

    for (plane, raw) in planes.iter_mut().zip(raw_planes) {
        let length = raw.truncate().length();
        *plane = if length > 0.0 { (raw / length).to_array() } else { raw.to_array() };
    }

    planes
}
