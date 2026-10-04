use super::Primitive;
use crate::AtlasUpdate;

// Primitives are drawn in order, each over the previous ones.
#[derive(Clone, Debug, Default)]
pub struct DrawList {
    pub primitives: Vec<Primitive>,
    pub atlas_updates: Vec<AtlasUpdate>,
}
