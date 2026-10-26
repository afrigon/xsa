use glam::Mat4;

/// The part of a mesh's index buffer one node draws, and where that node stands in the model's frame.
#[derive(Clone, PartialEq, Debug)]
pub struct Submesh {
    pub node: String,
    pub first_index: u32,
    pub index_count: u32,
    pub transform: Mat4,
}
