use winit::window::Window;

// Where a frame goes: presented to the window, or rendered into an image of the graph alone, for frames nobody sees
// while the compositor is not asking for any.
pub enum FrameTarget<'window> {
    Window { window: &'window Window },
    Offscreen,
}
