use kdl::KdlNode;

use crate::{FromNode, Id, NodeExtension, ParseContext};

// The material one of a model's mesh nodes draws with.
pub struct ModelMaterial {
    pub node: String,
    pub material: Id,
}

impl FromNode for ModelMaterial {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<ModelMaterial> {
        Ok(ModelMaterial {
            node: node.string_property("node")?.to_string(),
            material: context.parse_id(node.string_argument()?)?,
        })
    }
}
