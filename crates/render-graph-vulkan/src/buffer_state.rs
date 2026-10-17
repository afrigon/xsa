use ash::vk;

/// The stages and accesses a buffer was last used with, or must be made available to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferState {
    pub stages: vk::PipelineStageFlags2,
    pub access: vk::AccessFlags2,
}

impl BufferState {
    pub(crate) const UNUSED: BufferState = BufferState {
        stages: vk::PipelineStageFlags2::NONE,
        access: vk::AccessFlags2::NONE,
    };
}
