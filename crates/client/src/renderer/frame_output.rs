use winit::window::Window;

// The image a frame renders into, once a window frame acquired its swapchain image.
#[derive(Clone, Copy)]
pub(super) enum FrameOutput<'window> {
    Window { window: &'window Window, image_index: u32 },
    Offscreen,
}
