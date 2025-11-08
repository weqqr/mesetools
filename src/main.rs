use std::{error::Error, path::PathBuf};

use glam::ivec3;

use crate::world::{Map, SqliteBackend, WorldMeta};

pub mod world;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(world_path) = std::env::args().nth(1) else {
        eprintln!("world path required");
        std::process::exit(1);
    };

    let world_path = PathBuf::from(world_path);
    let world_meta_path = world_path.join("world.mt");

    let world_meta = WorldMeta::open(world_meta_path)?;

    let backend = world_meta.get_str("backend").unwrap();

    println!("backend: {backend}");

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

    let block = map.get_block(ivec3(0, 0, 0))?;

    Ok(())
}
