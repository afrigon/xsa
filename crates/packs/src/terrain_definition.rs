use std::path::PathBuf;

use anyhow::{Context, ensure};
use xsa_core::simulation::BodyDescription;

use crate::{BiomeDefinition, Document, Id, NodeExtension, PackData, PackStack, ParseContext};

const HEIGHT_MAP_EXTENSION: &str = "terrain";

pub struct TerrainDefinition {
    pub height_map: Option<PathBuf>,
    // The kinds of ground the body has, each with its file in `data/biomes/`.
    pub biomes: Vec<Id>,
}

impl TerrainDefinition {
    pub fn load_for_body(stack: &PackStack, body: &Id) -> anyhow::Result<Option<TerrainDefinition>> {
        let document = Document::read(stack.data(BodyDescription::KIND, body)?)?;
        let Some(node) = document.optional_node("terrain") else {
            return Ok(None);
        };
        let terrain = Id::parse(node.string_argument()?, &body.namespace)
            .with_context(|| format!("{}", document.path().display()))?;

        // A terrain no loaded pack provides leaves a smooth body, so a body can name terrain an optional pack adds.
        if let Err(err) = stack.data(TerrainDefinition::KIND, &terrain) {
            tracing::warn!("{body}: {err:#}");
            return Ok(None);
        }

        stack.load_data::<TerrainDefinition>(&terrain).map(Some)
    }
}

impl PackData for TerrainDefinition {
    const KIND: &'static str = "terrains";

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<TerrainDefinition> {
        let height_map = context.parse_id(document.node("height-map")?.string_argument()?)?;

        // A missing height map leaves a smooth body, so content loads before its generated files exist.
        let height_map = match context
            .stack
            .data_file(TerrainDefinition::KIND, &height_map, HEIGHT_MAP_EXTENSION)
        {
            Ok(path) => Some(path.to_path_buf()),
            Err(err) => {
                tracing::warn!("{}: {err:#}", context.id);
                None
            }
        };

        let biomes = document
            .nodes()
            .iter()
            .filter(|node| node.node_name() == "biome")
            .map(|node| {
                let biome = context.parse_id(node.string_argument()?)?;
                context.stack.load_data::<BiomeDefinition>(&biome)?;

                Ok(biome)
            })
            .collect::<anyhow::Result<Vec<Id>>>()?;
        ensure!(
            biomes.len() <= 1,
            "a terrain holds a single biome: placing several is not supported"
        );

        Ok(TerrainDefinition { height_map, biomes })
    }
}
