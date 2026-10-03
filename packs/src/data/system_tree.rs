use anyhow::{Context, bail, ensure};
use xsa_core::simulation::{BarycenterNode, BodyNode, SystemNode, SystemTree};

use crate::{Document, FromNode, NodeExtension, PackData, ParseContext};

impl PackData for SystemTree {
    const KIND: &'static str = "systems";

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<SystemTree> {
        let epoch = document.node("epoch")?.string_argument()?.parse()?;
        let mut roots = document.nodes().iter().filter(|node| node.node_name() != "epoch");
        let root = roots.next().context("a system needs a `star` or `barycenter` root")?;
        ensure!(roots.next().is_none(), "a system has exactly one root");

        let root = match root.node_name() {
            "star" => SystemNode::Body(Box::new(BodyNode::from_node(root, context)?)),
            "barycenter" => SystemNode::Barycenter(Box::new(BarycenterNode::from_node(root, context)?)),
            other => bail!("unknown system root `{other}`"),
        };

        Ok(SystemTree { epoch, root })
    }
}
