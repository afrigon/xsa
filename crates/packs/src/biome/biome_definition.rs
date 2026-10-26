use super::BiomeScatter;
use crate::{Document, FromNode, NodeExtension, PackData, ParseContext};

// What the server knows of a biome: the scatters it places.
pub struct BiomeDefinition {
    pub scatters: Vec<BiomeScatter>,
}

impl PackData for BiomeDefinition {
    const KIND: &'static str = "biomes";

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<BiomeDefinition> {
        let mut scatters = Vec::new();

        for node in document.nodes() {
            match node.node_name() {
                "scatter" => scatters.push(BiomeScatter::from_node(node, context)?),
                other => anyhow::bail!("unknown biome node `{other}`: use `scatter`"),
            }
        }

        Ok(BiomeDefinition { scatters })
    }
}
