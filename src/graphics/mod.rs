mod camera;
mod color;
mod renderer_status;
pub mod shape;
mod utils;

pub use self::camera::*;
pub use self::color::*;
pub use self::renderer_status::*;
pub(crate) use self::utils::*;
use glam::UVec2;
use std::sync::Arc;
use winit::window::Window;

#[derive(Debug)]
struct WgpuContextData {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

#[derive(Clone, Debug)]
pub struct WgpuContext(Arc<WgpuContextData>);

impl WgpuContext {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self(Arc::new(WgpuContextData { device, queue }))
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.0.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.0.queue
    }
}

#[derive(Debug)]
pub struct GraphicsContext {
    pub context: WgpuContext,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl GraphicsContext {
    #[inline]
    pub unsafe fn new(window: &Window) -> anyhow::Result<Self> {
        pollster::block_on(Self::new_async(window))
    }

    async unsafe fn new_async(window: &Window) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(Default::default());
        let surface = instance.create_surface(&window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or_else(|| anyhow::format_err!("Failed to get adapter"))?;

        let (device, queue) = adapter.request_device(&Default::default(), None).await?;

        let window_size = window.inner_size();
        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .ok_or_else(|| anyhow::format_err!("Failed to find a suitable surface format"))?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            context: WgpuContext::new(device, queue),
            surface,
            surface_config,
        })
    }

    pub fn resize_surface(&mut self, new_size: UVec2) {
        self.surface_config.width = new_size.x;
        self.surface_config.height = new_size.y;
        self.configure_surface();
    }

    pub fn configure_surface(&mut self) {
        self.surface
            .configure(self.context.device(), &self.surface_config)
    }
}
