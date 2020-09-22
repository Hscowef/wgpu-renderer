pub mod mesh;
pub mod texture;

use mesh::MeshManager;
use texture::TextureManager;

use std::sync::{Arc, Mutex};

pub struct RessourceManager {
    pub texture_manager: TextureManager,
    pub mesh_manager: MeshManager,
}

impl RessourceManager {
    pub fn new(device: Arc<Mutex<wgpu::Device>>, queue: Arc<Mutex<wgpu::Queue>>) -> Self {
        let texture_manager = TextureManager::new(device.clone(), queue.clone());
        let mesh_manager = MeshManager::new(device.clone());

        Self {
            texture_manager,
            mesh_manager,
        }
    }
}
