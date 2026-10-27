use super::MaterialHandle;

// The material one node of a model's mesh draws with.
pub struct NodeMaterial {
    pub node: String,
    pub material: MaterialHandle,
}
