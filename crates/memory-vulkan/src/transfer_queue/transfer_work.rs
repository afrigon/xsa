use std::sync::{Arc, Mutex};

use ash::vk;

use crate::transfer_queue::UploadDestination;
use crate::{Image, TimelineValue};

// The direction of a request and what it copies between.
pub(crate) enum TransferWork {
    Upload {
        data: Vec<Box<dyn AsRef<[u8]> + Send + Sync>>,
        destination: UploadDestination,
    },
    // Kept alive by the request until its last copy is recorded, then by the deletion queue.
    ImageToImage {
        source: Arc<Image>,
        destination: vk::Image,
        aspect: vk::ImageAspectFlags,
    },
    Readback {
        source: vk::Buffer,
        after: Option<TimelineValue>,
        result: Arc<Mutex<Vec<u8>>>,
    },
}
