use super::GlyphPrimitive;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Primitive {
    Glyph(GlyphPrimitive),
}
