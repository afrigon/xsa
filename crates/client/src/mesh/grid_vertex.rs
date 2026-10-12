#[repr(C)]
#[derive(Clone, Copy)]
pub struct GridVertex {
    pub cell: [u32; 2],
}
