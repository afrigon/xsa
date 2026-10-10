use crate::compiled_attachment::CompiledAttachment;
use crate::compiled_buffer_barrier::CompiledBufferBarrier;
use crate::compiled_image_barrier::CompiledImageBarrier;

#[derive(Clone, Default)]
pub(crate) struct CompiledStep {
    pub image_barriers: Vec<CompiledImageBarrier>,
    pub buffer_barriers: Vec<CompiledBufferBarrier>,
    pub attachments: Vec<CompiledAttachment>,
}
