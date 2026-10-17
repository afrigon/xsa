use std::sync::{Arc, Mutex};

use crate::transfer_queue::StagingSlice;

// Bytes a submitted batch copies into the readback ring, read into the request's result once the batch completes.
pub(crate) struct StagedReadback {
    pub slice: StagingSlice,
    pub size: usize,
    pub result: Arc<Mutex<Vec<u8>>>,
    pub result_offset: usize,
}
