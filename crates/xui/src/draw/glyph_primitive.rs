use crate::{AtlasRegion, LinearColor, Rect};

// `bounds` is in physical pixels and matches the atlas region's size one to one.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GlyphPrimitive {
    pub bounds: Rect,
    pub atlas_region: AtlasRegion,
    pub color: LinearColor,
}
