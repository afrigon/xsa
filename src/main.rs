mod app;
mod camera;
mod input;
mod mesh;
mod renderer;
mod simulation;
mod vulkan;

use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = app::App::default();
    event_loop.run_app(&mut app)?;
    app.into_result()
}
