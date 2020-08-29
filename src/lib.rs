use winit::{dpi::PhysicalSize, event::Event, event_loop::EventLoop, window::Window};
pub use winit::{event::WindowEvent, event_loop::ControlFlow, window::WindowBuilder};

/// Errors that occur while creating an [Application](Application).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApplicationCreationError {
    /// Building the window can fail because of
    /// denied permission, incompatible system, or lack of memory.
    BuildWindowError,
}

/// Struct used to initialize the renderer and to run the window.
/// It is created with the [Aplication::create](Application::create) function.
pub struct Application {
    event_loop: EventLoop<()>,
    window: Window,
    renderer: Renderer,
}

impl Application {
    /// Create an Application
    ///
    /// # Errors
    ///
    /// See [ApplicationCreationError](ApplicationCreationError) for the possible failure reasons.
    pub fn create(window_builder: WindowBuilder) -> Result<Self, ApplicationCreationError> {
        let event_loop = EventLoop::new();
        let window = window_builder
            .build(&event_loop)
            .map_err(|_| ApplicationCreationError::BuildWindowError)?;

        let renderer = futures::executor::block_on(Renderer::new(&window)).unwrap();

        Ok(Self {
            event_loop,
            window,
            renderer,
        })
    }

    /// # Warning
    ///
    /// Some [WindowEvent](WindowEvent) will never be passed to the `event_handler`:
    /// * [WindowEvent::CloseRequested](WindowEvent::CloseRequested)
    /// * [WindowEvent::Resized](WindowEvent::Resized)
    /// * [WindowEvent::ScaleFactorChanged](WindowEvent::ScaleFactorChanged)
    pub fn run<F>(self, mut event_hanlder: F) -> !
    where
        F: 'static + FnMut(WindowEvent, &mut ControlFlow),
    {
        let event_loop = self.event_loop;
        let window = self.window;
        let mut renderer = self.renderer;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => renderer.resize(new_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(*new_inner_size)
                    }
                    other_event => event_hanlder(other_event, control_flow),
                },
                Event::RedrawRequested(_) => renderer.render(),
                Event::MainEventsCleared => window.request_redraw(),
                _ => (),
            }
        });
    }
}

#[allow(dead_code)]
struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

impl Renderer {
    pub async fn new(window: &Window) -> Result<Self, ()> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
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

        let size = window.inner_size();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Ok(Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn render(&mut self) {
        let output_texture = self.swap_chain.get_current_frame().unwrap().output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_pass command encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        self.queue.submit(Some(encoder.finish()));
    }
}
