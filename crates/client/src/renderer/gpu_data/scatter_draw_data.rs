use super::GpuData;

// One submesh of a drawn scatter model: it draws every instance of its model's list.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterDrawData {
    pub list: u32,
    pub index_count: u32,
    pub first_index: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<ScatterDrawData>() == 16);

unsafe impl GpuData for ScatterDrawData {}
