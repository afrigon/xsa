use ash::vk;

/// The stages, accesses and layout an image was last used with, or must be left in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageState {
    pub stages: vk::PipelineStageFlags2,
    pub access: vk::AccessFlags2,
    pub layout: vk::ImageLayout,
}

impl ImageState {
    pub(crate) const UNUSED: ImageState = ImageState {
        stages: vk::PipelineStageFlags2::NONE,
        access: vk::AccessFlags2::NONE,
        layout: vk::ImageLayout::UNDEFINED,
    };
}
