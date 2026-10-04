use kdl::KdlNode;

use crate::ParseContext;

pub trait FromNode: Sized {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<Self>;
}
