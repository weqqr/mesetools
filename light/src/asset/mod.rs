use glam::{Vec2, Vec3};

pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub texcoord: Vec2,
}

#[derive(Default)]
pub struct Mesh {
    vertex_data: Vec<f32>,
    index_data: Vec<u32>,
    num_vertices: u32,
    num_indices: u32,
}

impl Mesh {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertex_data.extend_from_slice(&vertex.position.to_array());
        self.vertex_data.extend_from_slice(&vertex.normal.to_array());
        self.vertex_data.extend_from_slice(&vertex.texcoord.to_array());
        self.num_vertices += 1;
    }

    pub fn vertex_data(&self) -> &[f32] {
        &self.vertex_data
    }

    pub fn num_vertices(&self) -> u32 {
        self.num_vertices
    }
}
