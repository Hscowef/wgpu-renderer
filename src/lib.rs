pub mod geometry;
pub mod ressource_manager;

mod renderer;

pub use geometry::Vertex;

use renderer::Renderer;

pub use winit::window::WindowBuilder;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApplicationCreationError {
    BuildWindowError,
}

#[allow(dead_code)]
pub struct Application {
    event_loop: EventLoop<()>,
    window: Window,
    renderer: Renderer,
}

impl Application {
    pub fn create(window_builder: WindowBuilder) -> Result<Self, ApplicationCreationError> {
        let event_loop = EventLoop::new();
        let window = window_builder
            .build(&event_loop)
            .map_err(|_| ApplicationCreationError::BuildWindowError)?;

        let renderer = futures::executor::block_on(Renderer::create(&window)).unwrap();

        Ok(Self {
            event_loop,
            window,
            renderer,
        })
    }

    pub fn run(self) -> ! {
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
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    renderer.update();
                    renderer.render();
                }
                Event::MainEventsCleared => window.request_redraw(),
                _ => (),
            }
        });
    }
}
