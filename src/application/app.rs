use glam::vec3;
use sgpu::{AccessType, Format, GlobalBarrier, PipelineStage, QueueType, SgpuInititizationInfo, Swapchain, record, submit};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, WindowEvent},
    window::Window,
};

use crate::camera::Camera;
use crate::chunk::Face;
use crate::renderer::{FaceBuffer, IndirectDrawBuffer, IndirectDrawCommand, Renderer};
use crate::world::{ChunkUnloadInfo, WorkItem, WorkerPool, World};

use super::input::InputManager;

const MAX_FACES: usize = 8_000_000;
const MAX_COMMANDS: usize = 20_000;
const GENERATION_RADIUS: u32 = 8;
const UNLOAD_RADIUS: u32 = 10;
const NUM_WORKERS: usize = 8;
const SEED: u32 = 42;

struct PendingUnload {
    _coords: (i32, i32, i32),
    counter: sgpu::Counter,
    face_loc: crate::renderer::BufferLocation,
    cmd_slot: usize,
}

pub struct Application {
    swapchain: Swapchain,
    input_manager: InputManager,
    camera: Camera,
    renderer: Renderer,
    face_buffer: FaceBuffer,
    indirect_buffer: IndirectDrawBuffer,
    world: World,
    worker_pool: WorkerPool,
    size: PhysicalSize<u32>,
    pending_unloads: Vec<PendingUnload>,
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

        let face_buffer = FaceBuffer::new(MAX_FACES);
        let indirect_buffer = IndirectDrawBuffer::new(MAX_COMMANDS);
        let renderer = Renderer::new(size);
        let camera = Camera::new(vec3(0.0, 32.0, 0.0), size.width as f32 / size.height as f32);
        let mut world = World::new(GENERATION_RADIUS, UNLOAD_RADIUS);
        let mut worker_pool = WorkerPool::new(NUM_WORKERS, SEED);

        let (to_load, _) = world.update(0, 1, 0);
        for coords in to_load {
            worker_pool.submit(WorkItem { coords });
        }

        Application {
            swapchain,
            input_manager: InputManager::new(),
            camera,
            renderer,
            face_buffer,
            indirect_buffer,
            world,
            worker_pool,
            size,
            pending_unloads: Vec::new(),
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

    fn drain_pending(&mut self) {
        let mut i = 0;
        while i < self.pending_unloads.len() {
            if sgpu::poll(self.pending_unloads[i].counter) {
                let p = self.pending_unloads.swap_remove(i);
                self.face_buffer.free(p.face_loc);
                self.indirect_buffer.free_slot(p.cmd_slot);
            } else {
                i += 1;
            }
        }
    }

    fn poll_worker_results(&mut self, uploads: &mut Vec<sgpu::Counter>) {
        let mut results = Vec::new();
        while let Some(result) = self.worker_pool.try_recv() {
            results.push(result);
        }
        if results.is_empty() {
            return;
        }

        let mut cmd = record(QueueType::Transfer);
        for result in results {
            if result.mesh.faces.is_empty() {
                self.world.mark_loaded_empty(result.coords);
                continue;
            }

            let face_loc = self.face_buffer.allocate(result.mesh.faces.len());
            let cmd_slot = self.indirect_buffer.allocate_slot();

            let face_offset = (face_loc.offset / std::mem::size_of::<Face>() as u64) as u32;
            let draw_cmd = IndirectDrawCommand {
                vertex_count: result.mesh.faces.len() as u32 * 6,
                instance_count: 1,
                first_vertex: face_offset * 6,
                first_instance: 0,
                world_pos: [result.coords.0, result.coords.1, result.coords.2],
                _pad: 0.0,
            };

            cmd.update_buffer(&self.face_buffer.raw(), face_loc.offset, &result.mesh.faces);
            cmd.update_buffer(&self.indirect_buffer.raw(), self.indirect_buffer.slot_offset(cmd_slot), &[draw_cmd]);

            self.world.mark_loaded(result.coords, face_loc, cmd_slot);
        }

        uploads.push(submit(&[cmd]));
    }

    fn submit_unload_zeroes(&mut self, unloads: Vec<ChunkUnloadInfo>) {
        let zero_cmd = IndirectDrawCommand::zeroed();
        for info in unloads {
            let mut transfer_cmd = record(QueueType::Transfer);
            transfer_cmd.update_buffer(&self.indirect_buffer.raw(), self.indirect_buffer.slot_offset(info.cmd_slot), &[zero_cmd]);
            let counter = submit(&[transfer_cmd]);
            self.pending_unloads.push(PendingUnload {
                _coords: info.coords,
                counter,
                face_loc: info.face_loc,
                cmd_slot: info.cmd_slot,
            });
        }
    }

    fn wait_for_pending(&self, cmd: &mut sgpu::CommandBuffer, uploads: &[sgpu::Counter]) {
        for counter in uploads {
            cmd.wait_for(*counter, PipelineStage::ALL_COMMANDS);
        }
        for p in &self.pending_unloads {
            cmd.wait_for(p.counter, PipelineStage::ALL_COMMANDS);
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.camera.process_input(&self.input_manager, dt);
        self.input_manager.poll();

        let mut uploads = Vec::new();

        self.drain_pending();
        self.poll_worker_results(&mut uploads);

        let mut acquired = self.swapchain.acquire_image();
        let vp = self.camera.view_proj();

        let mut cmd = record(QueueType::Graphics);
        cmd.wait_for_swapchain_image(&acquired);

        self.wait_for_pending(&mut cmd, &uploads);

        cmd.global_barrier(&GlobalBarrier {
            previous_accesses: &[AccessType::TransferWrite],
            next_accesses: &[AccessType::VertexShaderStorageRead, AccessType::IndirectBuffer],
        });

        self.renderer.render(&mut cmd, acquired.image(), &self.face_buffer, &self.indirect_buffer, &vp, self.indirect_buffer.count() as u32);

        let submit_counter = submit(&[cmd]);
        self.swapchain.present(&mut acquired, submit_counter);
    }

    pub fn fixed_update(&mut self) {
        let pos = self.camera.position;
        let cx = (pos.x / 32.0) as i32;
        let cy = (pos.y / 32.0) as i32;
        let cz = (pos.z / 32.0) as i32;

        let (to_load, to_unload) = self.world.update(cx, cy, cz);

        for coords in to_load {
            self.worker_pool.submit(WorkItem { coords });
        }

        self.submit_unload_zeroes(to_unload);
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        sgpu::wait_idle();
    }
}
