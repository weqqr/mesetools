use std::{
    io::{Cursor, Read},
    sync::Mutex,
};

use glam::IVec3;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("block not found")]
    BlockNotFound,

    #[error("unsupported block version: {0}")]
    UnsupportedVersion(u8),

    #[error("unexpected line format: {0}")]
    UnexpectedFormat(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("postgres error: {0}")]
    Postgres(#[from] postgres::Error),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub struct Map {
    backend: Mutex<Box<dyn MapBackend>>,
}

impl Map {
    pub fn new(backend: impl MapBackend) -> Self {
        Self {
            backend: Mutex::new(Box::new(backend)),
        }
    }

    pub fn get_block(&self, pos: IVec3) -> Result<Block, Error> {
        let data = self.backend.lock().unwrap().get_block_data(pos)?;
        Block::parse_data(&data)
    }
}

pub trait MapBackend: 'static {
    fn get_block_data(&mut self, pos: IVec3) -> Result<Vec<u8>, Error>;
}

pub struct Block {
    data: Vec<u8>,
}

impl Block {
    pub fn parse_data(data: &[u8]) -> Result<Self, Error> {
        let mut cur = Cursor::new(data);
        let version = read_u8(&mut cur)?;

        if version < 29 {
            return Err(Error::UnsupportedVersion(version));
        }

        println!("{version}");

        Ok(Self { data: Vec::new() })
    }
}

fn read_u8(r: &mut impl Read) -> Result<u8, std::io::Error> {
    let mut buf = [0; 1];
    r.read_exact(&mut buf)?;

    Ok(buf[0])
}
