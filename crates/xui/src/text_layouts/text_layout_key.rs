use crate::Font;

// Everything that changes how a text is shaped, with floats as their bits so the key can be hashed.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct TextLayoutKey {
    pub content: String,
    pub family: String,
    pub size_bits: u32,
    pub weight_bits: u32,
    pub letter_spacing_bits: u32,
    pub features: Vec<String>,
    pub scale_factor_bits: u32,
}

impl TextLayoutKey {
    pub fn new(content: &str, font: &Font, scale_factor: f32) -> TextLayoutKey {
        TextLayoutKey {
            content: content.to_string(),
            family: font.family.clone(),
            size_bits: font.size.to_bits(),
            weight_bits: font.weight.to_bits(),
            letter_spacing_bits: font.letter_spacing.to_bits(),
            features: font.features.clone(),
            scale_factor_bits: scale_factor.to_bits(),
        }
    }
}
