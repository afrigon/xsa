use anyhow::{Context, bail, ensure};
use kdl::KdlNode;
use xsa_core::orbit::OrbitalElements;
use xsa_core::simulation::{BarycenterNode, BodyNode, SystemNode};

use crate::{FromNode, NodeExtension, ParseContext};

impl FromNode for BarycenterNode {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<BarycenterNode> {
        let name = node.string_argument()?.to_string();
        let mut orbit = None;
        let mut primary = None;
        let mut secondary = None;
        let mut children = Vec::new();

        for child in node.child_nodes() {
            match child.node_name() {
                "orbit" => orbit = Some(OrbitalElements::from_node(child, context)?),
                "primary" => primary = Some(BodyNode::from_node(child, context)?),
                "secondary" => secondary = Some(BodyNode::from_node(child, context)?),
                "body" => children.push(SystemNode::Body(Box::new(BodyNode::from_node(child, context)?))),
                "barycenter" => children.push(SystemNode::Barycenter(Box::new(BarycenterNode::from_node(
                    child, context,
                )?))),
                other => bail!("unknown node `{other}` in barycenter {name}"),
            }
        }

        let primary = primary.with_context(|| format!("barycenter {name} needs a `primary`"))?;
        let secondary = secondary.with_context(|| format!("barycenter {name} needs a `secondary`"))?;
        ensure!(
            primary.orbit.is_none(),
            "the primary of barycenter {name} has no orbit; the secondary's orbit is relative to it"
        );
        ensure!(
            secondary.orbit.is_some(),
            "the secondary of barycenter {name} needs an orbit relative to the primary"
        );

        Ok(BarycenterNode {
            name,
            orbit,
            primary,
            secondary,
            children,
        })
    }
}
