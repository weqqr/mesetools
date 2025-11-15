use std::collections::HashMap;


pub struct GlobalMapping {
    mapping: HashMap<String, u16>,
    last_id: u16,
}

impl GlobalMapping {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
            last_id: 0,
        }
    }

    pub fn get_or_insert_id(&mut self, name: &str) -> u16 {
        if let Some(id) = self.mapping.get(name).cloned() {
            return id;
        }

        let id = self.last_id;

        self.mapping.insert(name.to_string(), id);
        println!("{id} = {name}");

        self.last_id += 1;

        id
    }
}
