use anyhow::Context;

use super::{CurvatureBias, Handover, HandoverNoise, SurfaceSlot};
use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack};

const BIOMES: &str = "biomes";

// How a biome's ground looks up close: a surface for flat ground and one for slopes, where they hand over, and how the
// ground's shape moves the handover.
pub struct BiomeSurfacesDefinition {
    pub flat: SurfaceSlot,
    pub slope: SurfaceSlot,
    pub transition: Handover,
    pub handover_noise: HandoverNoise,
    pub curvature: CurvatureBias,
}

impl BiomeSurfacesDefinition {
    // None while the textures of one of its surfaces are missing.
    pub fn load(stack: &PackStack, biome: &Id) -> anyhow::Result<Option<BiomeSurfacesDefinition>> {
        let document = Document::read(stack.resource(BIOMES, biome, DOCUMENT_EXTENSION)?)?;

        BiomeSurfacesDefinition::parse(stack, &document, biome)
            .with_context(|| format!("{}", document.path().display()))
    }

    fn parse(stack: &PackStack, document: &Document, biome: &Id) -> anyhow::Result<Option<BiomeSurfacesDefinition>> {
        let slot = |name: &str| SurfaceSlot::load(stack, document.node(name)?, biome);
        let (Some(flat), Some(slope)) = (slot("flat")?, slot("slope")?) else {
            return Ok(None);
        };
        let handovers = document.node("handovers")?;
        let child = |name: &str| {
            handovers
                .child_nodes()
                .iter()
                .find(|node| node.node_name() == name)
                .with_context(|| format!("`handovers` needs a `{name}`"))
        };

        Ok(Some(BiomeSurfacesDefinition {
            flat,
            slope,
            transition: Handover::parse(child("transition")?)?,
            handover_noise: HandoverNoise::parse(child("noise")?)?,
            curvature: CurvatureBias::parse(document.node("curvature")?)?,
        }))
    }
}
