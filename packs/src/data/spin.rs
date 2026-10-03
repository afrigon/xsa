use kdl::KdlNode;
use xsa_core::simulation::Spin;

use crate::{FromNode, NodeExtension, ParseContext};

impl FromNode for Spin {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<Spin> {
        let radians =
            |key| -> anyhow::Result<f64> { Ok(node.optional_number_property(key)?.unwrap_or_default().to_radians()) };

        Ok(Spin {
            tidally_locked: node.bool_property("tidally-locked")?,
            azimuth: radians("spin-azimuth")?,
            prime_meridian: radians("prime-meridian")?,
        })
    }
}
