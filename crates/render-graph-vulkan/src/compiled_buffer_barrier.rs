use ash::vk;

use crate::buffer_barrier_source::BufferBarrierSource;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompiledBufferBarrier {
    pub buffer: usize,
    pub source: BufferBarrierSource,
    pub destination_stages: vk::PipelineStageFlags2,
    pub destination_access: vk::AccessFlags2,
}
