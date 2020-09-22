use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type TextureViewId = u32;
pub type SamplerId = u32;
pub type TextureId = u32;

pub struct TextureManager {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    device: Arc<Mutex<wgpu::Device>>,
    queue: Arc<Mutex<wgpu::Queue>>,
    texture_views: HashMap<TextureViewId, wgpu::TextureView>,
    samplers: HashMap<SamplerId, wgpu::Sampler>,
    textures: HashMap<TextureId, wgpu::BindGroup>,
    texture_view_id: TextureViewId,
    sampler_id: SamplerId,
    texture_id: TextureId,
}

impl TextureManager {
    pub fn new(device: Arc<Mutex<wgpu::Device>>, queue: Arc<Mutex<wgpu::Queue>>) -> Self {
        let mutex_guard = device.lock().unwrap();
        let bind_group_layout =
            mutex_guard.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture BindGroupLayout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
            });
        drop(mutex_guard);

        Self {
            bind_group_layout,
            device,
            queue,
            texture_views: HashMap::new(),
            samplers: HashMap::new(),
            textures: HashMap::new(),
            texture_view_id: 0,
            sampler_id: 0,
            texture_id: 0,
        }
    }

    pub(crate) fn bind_texture<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture: TextureId,
        idx: u32,
    ) {
        let bind_group = self
            .textures
            .get(&texture)
            .unwrap_or_else(|| panic!("Invalid TextureId: {}", texture));

        render_pass.set_bind_group(idx, bind_group, &[]);
    }

    pub fn create_texture_view(&mut self, bytes: &[u8]) -> TextureViewId {
        self.texture_view_id += 1;
        let image = image::load_from_memory(bytes).unwrap().to_rgba();
        let dimension = image.dimensions();
        let size = wgpu::Extent3d {
            width: dimension.0,
            height: dimension.1,
            depth: 1,
        };

        let device = self.device.lock().unwrap();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        drop(device);

        let queue = self.queue.lock().unwrap();
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * dimension.0,
                rows_per_image: dimension.1,
            },
            size,
        );
        drop(queue);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.texture_views.insert(self.texture_view_id, view);
        self.texture_view_id
    }

    pub fn create_sampler(&mut self, desc: &wgpu::SamplerDescriptor) -> SamplerId {
        let device = self.device.lock().unwrap();
        let sampler = device.create_sampler(desc);
        drop(device);
        self.sampler_id += 1;
        self.samplers.insert(self.sampler_id, sampler);
        self.sampler_id
    }

    pub fn create_texture(&mut self, texture_view: TextureViewId, sampler: SamplerId) -> TextureId {
        self.texture_id += 1;
        let view = self
            .texture_views
            .get(&texture_view)
            .unwrap_or_else(|| panic!("Invalid TextureViewId: {}", texture_view));
        let sampler = self
            .samplers
            .get(&sampler)
            .unwrap_or_else(|| panic!("Invalid SamplerId: {}", sampler));

        let device = self.device.lock().unwrap();
        let texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        drop(device);
        self.textures.insert(self.texture_id, texture);
        self.texture_id
    }

    pub fn drop_texture_view(&mut self, texture_view: TextureViewId) {
        self.texture_views.remove(&texture_view);
    }

    pub fn drop_sampler(&mut self, sampler: SamplerId) {
        self.samplers.remove(&sampler);
    }

    pub fn drop_texture(&mut self, texture: TextureId) {
        self.textures.remove(&texture);
    }
}
