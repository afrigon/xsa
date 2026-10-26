use resource_vulkan::{Mesh, ResourceHandle};

use super::NodeMaterial;

// A mesh with the material each of its nodes draws with. Drawn once its mesh is loaded and its nodes match its
// materials one for one.
pub struct Model {
    pub name: String,
    pub mesh: ResourceHandle<Mesh>,
    pub materials: Vec<NodeMaterial>,
}
