use anyhow::bail;
use kdl::KdlNode;
use xsa_core::orbit::OrbitalElements;
use xsa_core::simulation::{BarycenterNode, BodyDescription, BodyNode, Spin, SystemNode};

use crate::{FromNode, NodeExtension, ParseContext};

impl FromNode for BodyNode {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<BodyNode> {
        let id = context.parse_id(node.string_argument()?)?;
        let mut parsed = BodyNode {
            body: context.stack.load_data::<BodyDescription>(&id)?,
            orbit: None,
            spin: Spin::from_node(node, context)?,
            children: Vec::new(),
        };

        for child in node.child_nodes() {
            match child.node_name() {
                "orbit" => parsed.orbit = Some(OrbitalElements::from_node(child, context)?),
                "body" => parsed
                    .children
                    .push(SystemNode::Body(Box::new(BodyNode::from_node(child, context)?))),
                "barycenter" => parsed
                    .children
                    .push(SystemNode::Barycenter(Box::new(BarycenterNode::from_node(
                        child, context,
                    )?))),
                other => bail!("unknown node `{other}` in body {id}"),
            }
        }

        Ok(parsed)
    }
}
