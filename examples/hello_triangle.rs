use wgpu_renderer::Application;
use wgpu_renderer::WindowBuilder;

fn main() {
    let builder = WindowBuilder::new().with_title("Hello Triangle");
    let renderer = Application::create(builder).unwrap();
    renderer.run(|_, _| ());
}
