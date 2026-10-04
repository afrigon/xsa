use super::AtlasRegion;

// `left` and `top` place the bitmap relative to the glyph's pen position on the baseline.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AtlasGlyph {
    pub region: AtlasRegion,
    pub left: i32,
    pub top: i32,
}
