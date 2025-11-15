use std::error::Error;

use egui::ViewportId;
use render::Renderer;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::{Window, WindowId};

struct App {
    ctx: egui::Context,
    egui_winit_state: egui_winit::State,
    renderer: Option<Renderer>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let ctx = egui::Context::default();

        let egui_winit_state = egui_winit::State::new(
            ctx.clone(),
            ViewportId::ROOT,
            &event_loop.display_handle().unwrap(),
            None,
            None,
            None,
        );

        Self {
            ctx,
            egui_winit_state,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Forma")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = event_loop.create_window(window_attributes).unwrap();
        let renderer = Renderer::new(window);

        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Wait);

        if let Some(renderer) = &mut self.renderer {
            let _ = self
                .egui_winit_state
                .on_window_event(renderer.window(), &event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.render();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &mut self.renderer {
            let raw_input: egui::RawInput =
                self.egui_winit_state.take_egui_input(renderer.window());

            let full_output = self.ctx.run(raw_input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Hello world!");
                    if ui.button("Click me").clicked() {
                        println!("clicked");
                    }
                });
            });

            self.egui_winit_state
                .handle_platform_output(renderer.window(), full_output.platform_output);

            let clipped_primitives = self
                .ctx
                .tessellate(full_output.shapes, full_output.pixels_per_point);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app)?;

    Ok(())
}
