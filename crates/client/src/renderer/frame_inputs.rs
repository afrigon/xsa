use glam::Vec3;
use render_graph_vulkan::{ImportedBufferHandle, Subresource};

use super::passes::ScatterInputs;
use super::video_target::VideoTarget;

#[derive(Clone, Copy)]
pub(super) struct FrameInputs {
    pub output: Subresource,
    pub histogram: ImportedBufferHandle,
    pub capture: Option<ImportedBufferHandle>,
    pub video: Option<VideoTarget>,
    // None for a frame without scatter pages.
    pub scatter: Option<ScatterInputs>,
    pub projected_light_count: u32,
    pub lens_loaded: bool,
    // Linear.
    pub clear_color: Vec3,
}
