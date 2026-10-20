use crate::transfer_queue::CopyRegion;

// One copy: through staging, `size` bytes at `data_offset` in the request's byte array `data`; or `size` bytes
// between two images on the GPU, which stage nothing.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PieceCopy {
    pub data: usize,
    pub data_offset: usize,
    pub size: usize,
    pub alignment: usize,
    pub region: CopyRegion,
}
