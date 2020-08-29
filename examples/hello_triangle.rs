use wgpu_renderer::Application;
use winit::window::WindowBuilder;

fn main() {
    let builder = WindowBuilder::new().with_title("Hello Triangle");
    let renderer = Application::create(builder).unwrap();
    renderer.run();
}
