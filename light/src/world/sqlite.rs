use std::path::Path;

use rusqlite::Connection;

use crate::world::{Error, MapBackend};

pub struct SqliteBackend {
    conn: Connection,
}

impl SqliteBackend {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let conn = Connection::open(path)?;

        Ok(Self { conn })
    }
}

impl MapBackend for SqliteBackend {
    fn get_block_data(&mut self, pos: glam::IVec3) -> Result<Vec<u8>, Error> {
        const SQL: &str = "
            SELECT data
            FROM blocks
            WHERE x = ?
              AND y = ?
              AND z = ?
            LIMIT 1";

        let data = self
            .conn
            .query_one(SQL, [&pos.x, &pos.y, &pos.z], |row| row.get(0))?;

        Ok(data)
    }
}
