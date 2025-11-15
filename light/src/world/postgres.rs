use glam::IVec3;
use postgres::{Client, NoTls};

use crate::world::{Error, MapBackend};

pub struct PostgresBackend {
    client: Client,
}

impl PostgresBackend {
    pub fn new(dsn: String) -> Result<Self, Error> {
        let client = postgres::Client::connect(&dsn, NoTls)?;

        Ok(Self { client })
    }
}

impl MapBackend for PostgresBackend {
    fn get_block_data(&mut self, pos: IVec3) -> Result<Vec<u8>, Error> {
        const SQL: &str = "
            SELECT data
            FROM blocks
            WHERE posx = ?
              AND posy = ?
              AND posz = ?
            LIMIT 1";

        let row = self.client.query_one(SQL, &[&pos.x, &pos.y, &pos.z])?;
        let data = row.get(0);

        Ok(data)
    }
}
