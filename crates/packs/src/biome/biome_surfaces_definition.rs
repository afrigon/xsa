use anyhow::{Context, ensure};

use super::{CurvatureBias, Handover, HandoverNoise, SurfaceSlot};
use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack};

const BIOMES: &str = "biomes";

// How a biome's ground looks up close: a surface for flat ground, slopes and steep ground, where they hand over, and
// how the ground's shape moves the handovers.
pub struct BiomeSurfacesDefinition {
    pub flat: SurfaceSlot,
    pub slope: SurfaceSlot,
    pub steep: SurfaceSlot,
    pub flat_to_slope: Handover,
    pub slope_to_steep: Handover,
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
        let (Some(flat), Some(slope), Some(steep)) = (slot("flat")?, slot("slope")?, slot("steep")?) else {
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

        let flat_to_slope = Handover::parse(child("flat-to-slope")?)?;
        let slope_to_steep = Handover::parse(child("slope-to-steep")?)?;
        ensure!(
            flat_to_slope.angle + flat_to_slope.width / 2.0 <= slope_to_steep.angle - slope_to_steep.width / 2.0,
            "`flat-to-slope` must end before `slope-to-steep` starts"
        );

        Ok(Some(BiomeSurfacesDefinition {
            flat,
            slope,
            steep,
            flat_to_slope,
            slope_to_steep,
            handover_noise: HandoverNoise::parse(child("noise")?)?,
            curvature: CurvatureBias::parse(document.node("curvature")?)?,
        }))
    }
}
