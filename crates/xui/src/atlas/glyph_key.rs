#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct GlyphKey {
    pub font_id: u64,
    pub font_index: u32,
    pub glyph_id: u16,
    pub size_bits: u32,
    pub coordinates: Vec<i16>,
    pub subpixel_step: u8,
}
