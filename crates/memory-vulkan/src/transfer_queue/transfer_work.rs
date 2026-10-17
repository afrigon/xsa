use std::sync::{Arc, Mutex};

use ash::vk;

use crate::TimelineValue;
use crate::transfer_queue::UploadDestination;

// The direction of a request and what it copies between.
pub(crate) enum TransferWork {
    Upload {
        data: Vec<Vec<u8>>,
        destination: UploadDestination,
    },
    Readback {
        source: vk::Buffer,
        after: Option<TimelineValue>,
        result: Arc<Mutex<Vec<u8>>>,
    },
}
