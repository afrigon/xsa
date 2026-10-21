use crate::NodeExtension;

// A seeded noise moving the handovers across the ground, so class borders do not follow contour lines: its feature
// size (m) and how far it moves a handover (radians).
#[derive(Clone, Copy, Debug)]
pub struct HandoverNoise {
    pub size: f32,
    pub amplitude: f32,
}

impl HandoverNoise {
    pub(super) fn parse(node: &kdl::KdlNode) -> anyhow::Result<HandoverNoise> {
        Ok(HandoverNoise {
            size: node.number_property("size")? as f32,
            amplitude: (node.number_property("amplitude")? as f32).to_radians(),
        })
    }
}
