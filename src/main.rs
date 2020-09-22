//! This file is just for dev purpose only. There is not a stable api now
//! so I prefer this than adding an example.

use wgpu_renderer::Application;
use wgpu_renderer::WindowBuilder;

fn main() {
    let builder = WindowBuilder::new().with_title("Hello");
    let application = Application::create(builder).unwrap();
    application.run();
}
