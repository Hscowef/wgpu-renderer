use crate::geometry::Vertex;

use wgpu::util::DeviceExt;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type MeshId = u32;

pub struct MeshManager {
    device: Arc<Mutex<wgpu::Device>>,
    meshes: HashMap<MeshId, Mesh>,
    mesh_id: MeshId,
}

impl MeshManager {
    pub fn new(device: Arc<Mutex<wgpu::Device>>) -> Self {
        Self {
            device,
            meshes: HashMap::new(),
            mesh_id: 0,
        }
    }

    pub(crate) fn draw_mesh<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, mesh_id: MeshId) {
        let mesh = self
            .meshes
            .get(&mesh_id)
            .unwrap_or_else(|| panic!("Invalid MeshId: {}", mesh_id));

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        match &mesh.index_buffer {
            Some(index_buffer) => {
                render_pass.set_index_buffer(index_buffer.slice(..));
                render_pass.draw_indexed(0..mesh.nb_vertices, 0, 0..1);
            }
            None => render_pass.draw(0..mesh.nb_vertices, 0..1),
        };
    }

    pub fn create_mesh(&mut self, vertices: &[Vertex]) -> MeshId {
        self.mesh_id += 1;
        let nb_vertices = vertices.len() as u32;

        let device = self.device.lock().unwrap();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });
        drop(device);

        let mesh = Mesh {
            nb_vertices,
            vertex_buffer,
            index_buffer: None,
        };

        self.meshes.insert(self.mesh_id, mesh).unwrap();
        self.mesh_id
    }

    pub fn create_mesh_indexed(&mut self, vertices: &[Vertex], indices: &[u16]) -> MeshId {
        self.mesh_id += 1;
        let nb_vertices = indices.len() as u32;

        let device = self.device.lock().unwrap();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsage::INDEX,
        });
        drop(device);

        let mesh = Mesh {
            nb_vertices,
            vertex_buffer,
            index_buffer: Some(index_buffer),
        };

        self.meshes.insert(self.mesh_id, mesh);
        self.mesh_id
    }

    pub fn drop_mesh(&mut self, mesh_id: MeshId) {
        self.meshes.remove(&mesh_id);
    }
}

pub struct Mesh {
    nb_vertices: u32,
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
}
