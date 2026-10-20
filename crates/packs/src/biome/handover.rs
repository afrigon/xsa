use crate::NodeExtension;

// Where one class of ground gives way to the next as the slope grows: the slope at the middle of the handover and the
// range of slopes it spans, in radians.
#[derive(Clone, Copy, Debug)]
pub struct Handover {
    pub angle: f32,
    pub width: f32,
}

impl Handover {
    pub(super) fn parse(node: &kdl::KdlNode) -> anyhow::Result<Handover> {
        Ok(Handover {
            angle: (node.number_property("angle")? as f32).to_radians(),
            width: (node.number_property("width")? as f32).to_radians(),
        })
    }
}
