use std::{collections::HashMap, path::Path};

use crate::world::Error;

pub struct WorldMeta {
    values: HashMap<String, String>,
}

impl WorldMeta {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let data = std::fs::read_to_string(path)?;

        let mut values = HashMap::new();

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let (key, value) = line
                .split_once("=")
                .ok_or_else(|| Error::UnexpectedFormat(line.to_string()))?;

            values.insert(key.trim().to_string(), value.trim().to_string());
        }

        Ok(Self { values })
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }
}
