use crate::compiled_attachment::CompiledAttachment;
use crate::compiled_barrier::CompiledBarrier;
use crate::compiled_buffer_barrier::CompiledBufferBarrier;

#[derive(Clone, Default)]
pub(crate) struct CompiledStep {
    pub image_barriers: Vec<CompiledBarrier>,
    pub buffer_barriers: Vec<CompiledBufferBarrier>,
    pub attachments: Vec<CompiledAttachment>,
}
