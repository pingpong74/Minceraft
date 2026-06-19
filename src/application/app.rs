use glam::vec3;
use sgpu::{Format, QueueType, SgpuInititizationInfo, Swapchain, record, submit};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, WindowEvent},
    window::Window,
};

use crate::camera::Camera;
use crate::chunk::{CHUNK_SIDE, Chunk, Face, Generator, Neighbours, mesh};
use crate::renderer::{FaceBuffer, IndirectDrawBuffer, IndirectDrawCommand, Renderer};

use super::input::InputManager;

pub struct Application {
    swapchain: Swapchain,
    input_manager: InputManager,
    camera: Camera,
    renderer: Renderer,
    face_buffer: FaceBuffer,
    indirect_buffer: IndirectDrawBuffer,
    chunk_count: u32,
    size: PhysicalSize<u32>,
}

impl Application {
    pub fn new(window: &Window) -> Application {
        let size = window.inner_size();

        sgpu::sgpu_init(&SgpuInititizationInfo::default_from_window(window));
        let swapchain = sgpu::create_swapchain(
            window,
            &sgpu::SwapchainDescription {
                format: Format::Rgba16Float,
                frames_in_flight: 2,
                width: size.width,
                height: size.height,
            },
        );

        let mut face_buffer = FaceBuffer::new(1 << 24);
        let generator = Generator::new(42);
        let mut indirect_buffer = IndirectDrawBuffer::new(64);

        for cz in -3..=3 {
            for cy in -3..=3 {
                for cx in -3..=3 {
                    let blocks = generator.generate_blocks(cx * CHUNK_SIDE as i32, cy * CHUNK_SIDE as i32, cz * CHUNK_SIDE as i32);
                    let chunk = Chunk::new(blocks);
                    let mesh = mesh(
                        &chunk.blocks,
                        Neighbours {
                            xp: None,
                            xn: None,
                            yp: None,
                            yn: None,
                            zp: None,
                            zn: None,
                        },
                    );

                    if mesh.faces.len() == 0 {
                        continue;
                    }

                    let loc = face_buffer.allocate(&mesh.faces);
                    let first_face = (loc.offset / std::mem::size_of::<Face>() as u64) as u32;

                    indirect_buffer.allocate_command(&IndirectDrawCommand {
                        vertex_count: mesh.faces.len() as u32 * 6,
                        instance_count: 1,
                        first_vertex: first_face * 6,
                        first_instance: 0,
                        world_pos: [cx, cy, cz],
                        _pad: 0.0,
                    });
                }
            }
        }

        let chunk_count = indirect_buffer.count() as u32;

        Application {
            swapchain,
            input_manager: InputManager::new(),
            camera: Camera::new(vec3(0.0, 32.0, 0.0), size.width as f32 / size.height as f32),
            renderer: Renderer::new(size),
            face_buffer,
            indirect_buffer,
            chunk_count,
            size,
        }
    }

    pub fn handle_window_event(&mut self, window_event: &WindowEvent) {
        self.input_manager.handle_window_event(window_event);
    }

    pub fn handle_device_event(&mut self, device_event: &DeviceEvent) {
        self.input_manager.handle_device_event(device_event);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.swapchain.resize(size.width, size.height);
        self.renderer.resize(size);
        self.camera.resize(size);
        self.size = size;
    }

    pub fn update(&mut self, dt: f64) {
        self.camera.process_input(&self.input_manager, dt);
        self.input_manager.poll();

        let mut acquired = self.swapchain.acquire_image();

        let vp = self.camera.view_proj();

        let mut cmd = record(QueueType::Graphics);
        cmd.wait_for_swapchain_image(&acquired);

        self.renderer.render(&mut cmd, acquired.image(), &mut self.face_buffer, &mut self.indirect_buffer, &vp, self.chunk_count);

        let counter = submit(&[cmd]);
        self.swapchain.present(&mut acquired, counter);
    }

    pub fn fixed_update(&self) {}
}
