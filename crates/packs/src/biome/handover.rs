use anyhow::ensure;

use crate::NodeExtension;

// Where one class of ground gives way to the next as the slope grows: the slope at the middle of the handover and the
// range of slopes it spans, in radians, and how sharply the higher of the two surfaces shows through where they blend:
// from 0, a cross-fade over the whole displacement range, toward 1, only the higher surface.
#[derive(Clone, Copy, Debug)]
pub struct Handover {
    pub angle: f32,
    pub width: f32,
    pub blend_sharpness: f32,
}

impl Handover {
    pub(super) fn parse(node: &kdl::KdlNode) -> anyhow::Result<Handover> {
        let blend_sharpness = node.number_property("blend-sharpness")? as f32;
        ensure!(
            (0.0..1.0).contains(&blend_sharpness),
            "`blend-sharpness` runs from 0 up to, but not including, 1"
        );

        Ok(Handover {
            angle: (node.number_property("angle")? as f32).to_radians(),
            width: (node.number_property("width")? as f32).to_radians(),
            blend_sharpness,
        })
    }
}
