use render_graph_vulkan::{ImageHandle, Subresource};

// Where a captured frame renders: an image of its own the video reads, copied to the window when it shows one.
#[derive(Clone, Copy)]
pub(super) struct VideoTarget {
    pub frame: ImageHandle,
    pub window: Option<Subresource>,
}
