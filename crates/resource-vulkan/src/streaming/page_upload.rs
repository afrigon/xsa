use std::sync::Arc;

use memory_vulkan::{GpuUpload, Image};

use crate::streaming::{CellAddress, PageOwner};

// A page copying into a cell. Dropping it cancels the copy if it has not started.
pub(crate) struct PageUpload {
    pub owner: PageOwner,
    pub cell: CellAddress,
    pub upload: GpuUpload<Arc<Image>>,
}
