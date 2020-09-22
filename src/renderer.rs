use crate::geometry::Vertex;
use crate::ressource_manager::RessourceManager;
use crate::Window;

use winit::dpi::PhysicalSize;

use std::sync::{Arc, Mutex};

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex::new([-0.0868241, 0.49240386, 0.0], [0.4131759, 0.00759614],),
    Vertex::new([-0.49513406, 0.06958647, 0.0], [0.0048659444, 0.43041354],), 
    Vertex::new([-0.21918549, -0.44939706, 0.0], [0.28081453, 0.949397057],),
    Vertex::new([0.35966998, -0.3473291, 0.0], [0.85967, 0.84732911],), 
    Vertex::new([0.44147372, 0.2347359, 0.0], [0.9414737, 0.2652641],),
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub struct Renderer {
    surface: wgpu::Surface,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,

    // Providing thread safety has no use for now but I think it may become handy later.
    device: Arc<Mutex<wgpu::Device>>,
    queue: Arc<Mutex<wgpu::Queue>>,
    ressource_manager: RessourceManager,

    // The followings are for testing purpose only
    mesh_id: u32,
    texture_id: u32,
}

impl Renderer {
    pub async fn create(window: &Window) -> Result<Self, ()> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (temp_device, temp_queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(Mutex::new(temp_device));
        let queue = Arc::new(Mutex::new(temp_queue));

        let mut ressource_manager = RessourceManager::new(device.clone(), queue.clone());

        let mesh_id = ressource_manager
            .mesh_manager
            .create_mesh_indexed(VERTICES, INDICES);

        let texture_bytes = include_bytes!("../happy-tree.png");
        let texture_view_id = ressource_manager
            .texture_manager
            .create_texture_view(texture_bytes);

        let sampler_id =
            ressource_manager
                .texture_manager
                .create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

        let texture_id = ressource_manager
            .texture_manager
            .create_texture(texture_view_id, sampler_id);

        let lock_device = device.lock().unwrap();
        let lock_queue = queue.lock().unwrap();

        let size = window.inner_size();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = lock_device.create_swap_chain(&surface, &sc_desc);

        let render_pipeline_layout =
            lock_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[&ressource_manager.texture_manager.bind_group_layout],
                push_constant_ranges: &[],
            });

        let vs_module =
            lock_device.create_shader_module(wgpu::include_spirv!("../shader.vert.spv"));
        let fs_module =
            lock_device.create_shader_module(wgpu::include_spirv!("../shader.frag.spv"));
        let render_pipeline = lock_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::BUFFER_DESCRIPTOR],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: true,
        });

        drop(lock_device);
        drop(lock_queue);

        Ok(Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,

            mesh_id,
            texture_id,

            ressource_manager,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let device = self.device.lock().unwrap();

        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn render(&mut self) {
        let device = self.device.lock().unwrap();

        let output_texture = self.swap_chain.get_current_frame().unwrap().output;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_pass command encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &output_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            self.ressource_manager.texture_manager.bind_texture(
                &mut render_pass,
                self.texture_id,
                0,
            );
            self.ressource_manager
                .mesh_manager
                .draw_mesh(&mut render_pass, self.mesh_id);
        }

        let queue = self.queue.lock().unwrap();
        queue.submit(Some(encoder.finish()));
    }

    pub fn update(&mut self) {}
}
