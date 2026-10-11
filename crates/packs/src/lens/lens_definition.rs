use anyhow::{Context, bail};

use super::{GhostDefinition, GlareDefinition, OffAxisFalloffDefinition, StarburstDefinition};
use crate::{DOCUMENT_EXTENSION, Document, FromNode, Id, NodeExtension, PackStack, ParseContext};

const LENSES: &str = "lenses";

pub struct LensDefinition {
    pub glare: GlareDefinition,
    pub starburst: StarburstDefinition,
    pub off_axis_falloff: OffAxisFalloffDefinition,
    pub ghosts: Vec<GhostDefinition>,
}

impl LensDefinition {
    pub fn load(stack: &PackStack, id: &Id) -> anyhow::Result<LensDefinition> {
        let document = Document::read(stack.resource(LENSES, id, DOCUMENT_EXTENSION)?)?;
        let context = ParseContext { stack, id };

        LensDefinition::parse(&document, &context).with_context(|| format!("{}", document.path().display()))
    }

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<LensDefinition> {
        let mut ghosts = Vec::new();

        for node in document.nodes() {
            match node.node_name() {
                "off-axis-falloff" | "glare" | "starburst" => {}
                "ghost" => ghosts.push(GhostDefinition::from_node(node, context)?),
                other => bail!("unknown lens node `{other}`: use `off-axis-falloff`, `glare`, `starburst` or `ghost`"),
            }
        }

        Ok(LensDefinition {
            glare: GlareDefinition::from_node(document.node("glare")?, context)?,
            starburst: StarburstDefinition::from_node(document.node("starburst")?, context)?,
            off_axis_falloff: OffAxisFalloffDefinition::from_node(document.node("off-axis-falloff")?, context)?,
            ghosts,
        })
    }
}
