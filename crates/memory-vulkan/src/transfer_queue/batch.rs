use std::time::Instant;

use ash::vk;

use crate::transfer_queue::{StagedReadback, TransferCompletion};

// A submitted batch of copies, finished once the transfer timeline reaches `value`.
pub(crate) struct Batch {
    pub value: u64,
    pub command_buffer: vk::CommandBuffer,
    pub bytes: u64,
    pub submitted_at: Instant,
    pub upload_blocks: Vec<usize>,
    pub readback_blocks: Vec<usize>,
    pub readbacks: Vec<StagedReadback>,
    pub completions: Vec<Box<dyn TransferCompletion>>,
}
