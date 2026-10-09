use ash::vk;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompiledBufferBarrier {
    pub buffer: usize,
    pub source_stages: vk::PipelineStageFlags2,
    pub source_access: vk::AccessFlags2,
    pub destination_stages: vk::PipelineStageFlags2,
    pub destination_access: vk::AccessFlags2,
}
