use glam::UVec2;
use winit::window::Window;

#[derive(Debug)]
pub struct Graphics {
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Graphics {
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
            surface,
            surface_config,
            device,
            queue,
        })
    }

    pub fn resize_surface(&mut self, new_size: UVec2) {
        self.surface_config.width = new_size.x;
        self.surface_config.height = new_size.y;
        self.configure_surface();
    }

    pub fn configure_surface(&mut self) {
        self.surface.configure(&self.device, &self.surface_config)
    }
}
