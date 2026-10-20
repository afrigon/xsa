use std::sync::Arc;

use memory_vulkan::{GpuUpload, Image};

use crate::streaming::{CellAddress, PageOwner};

// A resident page copying out of a draining chunk into `destination`, on the GPU.
pub(crate) struct PageMove {
    pub owner: PageOwner,
    pub destination: CellAddress,
    pub copy: GpuUpload<Arc<Image>>,
}
