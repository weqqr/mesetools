#![allow(clippy::new_without_default)]
#![allow(clippy::single_match)]

use std::{error::Error, path::PathBuf};

use glam::{Vec3, vec3};
use winit::event::{DeviceEvent, DeviceId};
use winit::event_loop::ControlFlow;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::camera::Camera;
use crate::input::Input;
use crate::{
    render::Renderer,
    world::{Map, SqliteBackend, WorldMeta},
};

pub mod asset;
pub mod camera;
pub mod input;
pub mod render;
pub mod world;

struct App {
    renderer: Option<Renderer>,
    camera: Camera,
    input: Input,
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,
            camera: Camera::new(),
            input: Input::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Light");
        let window = event_loop.create_window(window_attributes).unwrap();
        let renderer = Renderer::new(window);
        let adapter_info = renderer.adapter_info();

        renderer.window().set_title(&format!(
            "Light ({} on {})",
            adapter_info.backend, adapter_info.name
        ));

        self.renderer = Some(renderer)
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Poll);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::KeyboardInput { ref event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    event_loop.exit();
                }
            }
            _ => {}
        }

        self.input.submit_event(&event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.input.submit_device_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        let (forward, right) = self.camera.forward_right();
        let speed = 0.1;

        let mut movement_delta = Vec3::ZERO;

        if self.input.is_key_pressed(KeyCode::KeyW) {
            movement_delta += forward;
        }

        if self.input.is_key_pressed(KeyCode::KeyS) {
            movement_delta -= forward;
        }

        if self.input.is_key_pressed(KeyCode::KeyA) {
            movement_delta -= right;
        }

        if self.input.is_key_pressed(KeyCode::KeyD) {
            movement_delta += right;
        }

        if self.input.is_key_pressed(KeyCode::Space) {
            movement_delta += Vec3::Y;
        }

        if self.input.is_key_pressed(KeyCode::ShiftLeft) {
            movement_delta -= Vec3::Y;
        }

        self.camera.position += movement_delta.normalize_or_zero() * speed;

        let sensitivity = 0.1;
        let mouse_delta = self.input.mouse_delta() * sensitivity;
        self.camera.rotate(mouse_delta.y, mouse_delta.x);
        self.input.reset_mouse_delta();

        renderer.render(&self.camera);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(world_path) = std::env::args().nth(1) else {
        eprintln!("world path required");
        std::process::exit(1);
    };

    let world_path = PathBuf::from(world_path);
    let world_meta_path = world_path.join("world.mt");

    let world_meta = WorldMeta::open(world_meta_path)?;

    let backend = world_meta.get_str("backend").unwrap();

    let map = match backend {
        "sqlite3" => {
            let sqlite_path = world_path.join("map.sqlite");
            let sqlite = SqliteBackend::new(sqlite_path)?;
            Map::new(sqlite)
        }
        "postgres" => {
            unimplemented!()
        }
        _ => {
            eprintln!("unknown backend: {backend}");
            std::process::exit(1);
        }
    };

    let event_loop = EventLoop::new()?;
    let mut app = App::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}
