use std::sync::Arc;

use winit::window::Window;

#[derive(Clone)]
pub struct GpuContext {
    window: Arc<Window>,
}

impl GpuContext {
    pub fn new(window: Arc<Window>) -> anyhow::Result<GpuContext> {
        return Ok(GpuContext { window: window });
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {}
}

#[derive(Clone)]
pub struct RenderContext {}

impl RenderContext {
    pub fn new() -> anyhow::Result<RenderContext> {
        return Ok(RenderContext {});
    }
}

#[derive(Clone)]
pub struct Renderer {
    gpu_context: Arc<GpuContext>,
    render_context: Arc<RenderContext>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
        let rcx = Arc::new(RenderContext::new().expect("Failed to create render contex"));
        let gpu_context = Arc::new(GpuContext::new(window).expect("Failed to make gpu context"));

        // need to impl
        return Ok(Renderer {
            render_context: rcx,
            gpu_context: gpu_context,
        });
    }

    pub fn render(&mut self) {}
}
