use pollster::FutureExt;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer {
    surface: Surface<'static>,
    adapter: Adapter,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,

    window: Window,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        let instance = Instance::new(&InstanceDescriptor::default());

        // SAFETY: Window has the same lifetime as surface
        let surface = unsafe {
            instance
                .create_surface_unsafe(SurfaceTargetUnsafe::from_window(&window).unwrap())
                .unwrap()
        };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .unwrap();

        let inner_size = window.inner_size();
        let mut surface_config = surface
            .get_default_config(&adapter, inner_size.width, inner_size.height)
            .unwrap();
        surface_config.present_mode = PresentMode::AutoNoVsync;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .block_on()
            .unwrap();

        let mut renderer = Self {
            surface,
            adapter,
            surface_config,
            device,
            queue,

            window,
        };

        renderer.resize(inner_size);

        renderer
    }

    pub fn adapter_info(&self) -> AdapterInfo {
        self.adapter.get_info()
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface_config.width = size.width;
        self.surface_config.height = size.height;

        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn render(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let surface_texture = self.surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        {
            let _rp = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit([encoder.finish()]);

        surface_texture.present();
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
