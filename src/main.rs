use std::{error::Error, path::PathBuf};

use glam::{vec2, vec3};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::asset::{Mesh, Vertex};
use crate::render::MeshBuffer;
use crate::{
    render::Renderer,
    world::{Map, SqliteBackend, WorldMeta},
};

pub mod asset;
pub mod render;
pub mod world;

struct App {
    renderer: Option<Renderer>,
    mesh_buffer: Option<MeshBuffer>,
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,
            mesh_buffer: None,
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

        let mut triangle = Mesh::new();
        triangle.add_vertex(Vertex {
            position: vec3(-0.5, -0.5, 0.0),
            normal: vec3(0.0, 0.0, 1.0),
            texcoord: vec2(0.0, 0.0),
        });
        triangle.add_vertex(Vertex {
            position: vec3(0.5, -0.5, 0.0),
            normal: vec3(0.0, 0.0, 1.0),
            texcoord: vec2(1.0, 0.0),
        });
        triangle.add_vertex(Vertex {
            position: vec3(0.0, 0.5, 0.0),
            normal: vec3(0.0, 0.0, 1.0),
            texcoord: vec2(0.5, 1.0),
        });

        self.mesh_buffer = Some(renderer.create_mesh_buffer(&triangle));

        self.renderer = Some(renderer)
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        let Some(mesh_buffer) = &mut self.mesh_buffer else {
            return;
        };

        renderer.render(&mesh_buffer);
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
