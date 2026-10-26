use crate::renderer::{MaterialHandle, ModelHandle};

// One submesh of a scatter model, drawn with indirect arguments the frame's draws step writes.
pub(in crate::renderer) struct ScatterMeshDraw {
    pub model: ModelHandle,
    pub submesh: usize,
    pub material: MaterialHandle,
}
