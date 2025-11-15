#![allow(clippy::new_without_default)]
#![allow(clippy::single_match)]

use std::{error::Error, path::PathBuf};

use glam::{Vec3, ivec3};
use winit::dpi::PhysicalSize;
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
use crate::node::GlobalMapping;
use crate::render::DataBuffer;
use crate::world::Block;
use crate::{
    render::Renderer,
    world::{Map, SqliteBackend, WorldMeta},
};

pub mod asset;
pub mod camera;
pub mod input;
pub(crate) mod node;
pub mod render;
pub mod world;

struct App {
    renderer: Option<Renderer>,
    camera: Camera,
    input: Input,
    map: Map,
    global_mapping: GlobalMapping,
    grid: Option<DataBuffer>,
}

impl App {
    pub fn new(map: Map) -> Self {
        Self {
            renderer: None,
            camera: Camera::new(),
            input: Input::new(),
            map,
            global_mapping: GlobalMapping::new(),
            grid: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Light")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = event_loop.create_window(window_attributes).unwrap();
        let renderer = Renderer::new(window);

        let adapter_info = renderer.adapter_info();
        renderer.window().set_title(&format!(
            "Light ({} on {})",
            adapter_info.backend, adapter_info.name
        ));

        let air_id = self.global_mapping.get_or_insert_id("air");
        assert_eq!(air_id, 0);

        let block = self.map.get_block(ivec3(0, 2, 0)).unwrap();
        let grid = block_to_grid(&block, &mut self.global_mapping);
        let grid = renderer.create_data_buffer(bytemuck::cast_slice(&grid));

        self.renderer = Some(renderer);
        self.grid = Some(grid);
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

        let Some(grid) = &self.grid else {
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

        renderer.render(&self.camera, grid);
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
    let mut app = App::new(map);

    event_loop.run_app(&mut app)?;

    Ok(())
}

fn block_to_grid(block: &Block, global_mapping: &mut GlobalMapping) -> Vec<u32> {
    let mut data = vec![0; 16 * 16 * 16];

    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let node = block.get_node(ivec3(x, y, z));
                let name = block.get_name_by_id(node.id).unwrap();
                let global_id = global_mapping.get_or_insert_id(name);

                let mut value = 0;
                value |= (global_id as u32) << 16;
                value |= node.param1 as u32;
                value |= node.param2 as u32;

                let index = (z * 16 * 16 + y * 16 + x) as usize;
                data[index] = value;
            }
        }
    }

    data
}
