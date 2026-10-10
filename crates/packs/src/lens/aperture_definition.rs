use anyhow::ensure;
use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

const MINIMUM_BLADES: f64 = 3.0;

pub struct ApertureDefinition {
    pub blades: u32,
    pub rotation_degrees: f32,
}

impl FromNode for ApertureDefinition {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<ApertureDefinition> {
        let blades = node.number_property("blades")?;
        ensure!(
            blades.fract() == 0.0 && blades >= MINIMUM_BLADES,
            "`aperture` property `blades` must be a whole number of at least {MINIMUM_BLADES}"
        );

        Ok(ApertureDefinition {
            blades: blades as u32,
            rotation_degrees: node.number_property("rotation")? as f32,
        })
    }
}
