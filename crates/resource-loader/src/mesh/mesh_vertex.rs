use super::TangentFrame;
use crate::utils::HalfFloat;

const MESH_VERTEX_BYTES: usize = 24;

/// A mesh vertex as the GPU reads it: its position in its node's frame, its tangent frame and its texture coordinates.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub tangent_frame: TangentFrame,
    pub uv: [HalfFloat; 2],
}

const _: () = assert!(size_of::<MeshVertex>() == MESH_VERTEX_BYTES);
