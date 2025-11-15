use std::{
    collections::HashMap,
    io::{Cursor, Read},
    string::FromUtf8Error,
    sync::Mutex,
};

use glam::IVec3;

// TODO: split this
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("block not found")]
    BlockNotFound,

    #[error("unsupported block version: {0}")]
    UnsupportedVersion(u8),

    #[error("unexpected line format: {0}")]
    UnexpectedFormat(String),

    #[error("invalid utf-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

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
    node_data: Vec<u8>,
    mappings: HashMap<u16, String>,
}

pub struct Node {
    pub id: u16,
    pub param1: u8,
    pub param2: u8,
}

impl Block {
    const VOLUME: usize = 16 * 16 * 16;

    pub fn parse_data(data: &[u8]) -> Result<Self, Error> {
        let mut cur = Cursor::new(data);
        let version = read_u8(&mut cur)?;

        if version < 29 {
            return Err(Error::UnsupportedVersion(version));
        }

        let mut decoder = zstd::Decoder::new(&mut cur)?;

        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;

        let mut cur = Cursor::new(buf);
        let _flags = read_u8(&mut cur)?;
        let _lighting_complete = read_u16(&mut cur)?;
        let _timestamp = read_u32(&mut cur)?;
        let _mapping_version = read_u8(&mut cur)?;

        let mappings_count = read_u16(&mut cur)?;

        let mut mappings = HashMap::new();

        for _ in 0..mappings_count {
            let id = read_u16(&mut cur)?;
            let name = read_string(&mut cur)?;

            mappings.insert(id, name);
        }

        let _content_width = read_u8(&mut cur);
        let _params_width = read_u8(&mut cur);

        let mut node_data = vec![0; Self::VOLUME * 4];
        cur.read_exact(&mut node_data)?;

        Ok(Self {
            node_data,
            mappings,
        })
    }

    pub fn get_name_by_id(&self, id: u16) -> Option<&str> {
        self.mappings.get(&id).map(|s| s.as_str())
    }

    pub fn get_node(&self, pos: IVec3) -> Node {
        let node_index = Self::node_index(pos);

        let id_hi = self.node_data[2 * node_index] as u16;
        let id_lo = self.node_data[2 * node_index + 1] as u16;
        let param1 = self.node_data[Self::VOLUME * 2 + node_index];
        let param2 = self.node_data[Self::VOLUME * 3 + node_index];

        Node {
            id: (id_hi << 8) | id_lo,
            param1,
            param2,
        }
    }

    fn node_index(pos: IVec3) -> usize {
        assert!(pos.x >= 0 && pos.x < 16);
        assert!(pos.y >= 0 && pos.y < 16);
        assert!(pos.z >= 0 && pos.z < 16);

        pos.z as usize * 16 * 16 + pos.y as usize * 16 + pos.x as usize
    }
}

fn read_u8(r: &mut impl Read) -> Result<u8, std::io::Error> {
    let mut buf = [0; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16(r: &mut impl Read) -> Result<u16, std::io::Error> {
    let mut buf = [0; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32(r: &mut impl Read) -> Result<u32, std::io::Error> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_string(r: &mut impl Read) -> Result<String, Error> {
    let len = read_u16(r)?;
    let mut data = vec![0; len as usize];
    r.read_exact(&mut data)?;
    let string = String::from_utf8(data)?;
    Ok(string)
}
