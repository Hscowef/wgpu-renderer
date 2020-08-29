use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApplicationCreationError {
    BuildWindowError,
}

pub struct Application {
    event_loop: EventLoop<()>,
    window: Window,
}

impl Application {
    pub fn create(window_builder: WindowBuilder) -> Result<Self, ApplicationCreationError> {
        let event_loop = EventLoop::new();
        let window = window_builder
            .build(&event_loop)
            .map_err(|_| ApplicationCreationError::BuildWindowError)?;

        Ok(Self { event_loop, window })
    }

    pub fn run(self) -> ! {
        let event_loop = self.event_loop;
        let window = self.window;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::MainEventsCleared => window.request_redraw(),
                Event::RedrawRequested(_) => {}
                _ => (),
            }
        });
    }
}
