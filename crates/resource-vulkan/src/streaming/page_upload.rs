use std::sync::Arc;

use memory_vulkan::{GpuUpload, Image};

use crate::streaming::{LayerAddress, PageOwner};

// A page copying into a layer. Dropping it cancels the copy if it has not started.
pub(crate) struct PageUpload {
    pub owner: PageOwner,
    pub layer: LayerAddress,
    pub upload: GpuUpload<Arc<Image>>,
}
