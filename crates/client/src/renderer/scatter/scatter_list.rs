// The instances of one model drawn this frame, from `first_instance` in the frame's instance buffer: room for every
// instance of its resident pages.
pub(in crate::renderer) struct ScatterList {
    pub first_instance: u32,
    pub triangles_per_instance: u64,
}
