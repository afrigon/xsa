use ash::vk;

/// The stages and accesses a buffer was last used with, or must be made available to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferState {
    pub stages: vk::PipelineStageFlags2,
    pub access: vk::AccessFlags2,
}
