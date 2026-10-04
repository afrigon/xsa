/// Where one frame's interface is recorded: a command buffer in the recording state and the color image it draws
/// over, `width` by `height` pixels. `frame_slot` is below the renderer's frames in flight.
pub struct FrameTarget {
    pub command_buffer: u64,
    pub image_view: u64,
    pub width: u32,
    pub height: u32,
    pub frame_slot: usize,
}
