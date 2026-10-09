use ash::vk;

#[derive(Clone, Copy)]
pub(crate) struct CompiledAttachment {
    pub image: usize,
    pub level: u32,
    pub depth: bool,
    pub layout: vk::ImageLayout,
    pub load_op: vk::AttachmentLoadOp,
    pub store_op: vk::AttachmentStoreOp,
    pub clear_value: vk::ClearValue,
}
