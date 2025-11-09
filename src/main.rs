use std::{error::Error, path::PathBuf};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    render::Renderer,
    world::{Map, SqliteBackend, WorldMeta},
};

pub mod render;
pub mod world;

struct App {
    renderer: Option<Renderer>,
}

impl App {
    pub fn new() -> Self {
        Self { renderer: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Light");
        let window = event_loop.create_window(window_attributes).unwrap();
        let renderer = Renderer::new(window);

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
            _ => {}
        }
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
