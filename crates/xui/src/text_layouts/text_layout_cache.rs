use std::collections::HashMap;

use parley::{FontFamily, FontWeight, LayoutContext, StyleProperty};

use super::{CachedTextLayout, TextLayoutKey};
use crate::FontLibrary;

const OPTICAL_SIZE_AXIS: &str = "opsz";

// Shaping is the costly part of text layout, and layout measures a text many times per frame: each text is shaped
// once and kept for as long as some frame uses it.
pub(crate) struct TextLayoutCache {
    layouts: HashMap<TextLayoutKey, CachedTextLayout>,
    frame: u64,
}

impl TextLayoutCache {
    pub fn new() -> TextLayoutCache {
        TextLayoutCache {
            layouts: HashMap::new(),
            frame: 0,
        }
    }

    pub fn get(
        &mut self,
        key: TextLayoutKey,
        fonts: &mut FontLibrary,
        builders: &mut LayoutContext<()>,
    ) -> &mut CachedTextLayout {
        let frame = self.frame;
        let layout = self
            .layouts
            .entry(key)
            .or_insert_with_key(|key| CachedTextLayout::new(TextLayoutCache::shape(key, fonts, builders), frame));
        layout.last_used = frame;

        layout
    }

    // Drops the texts the frame that just ended did not use.
    pub fn end_frame(&mut self) {
        let frame = self.frame;
        self.layouts.retain(|_, layout| layout.last_used == frame);
        self.frame += 1;
    }

    // In physical pixels: parley scales every metric by the scale factor.
    fn shape(key: &TextLayoutKey, fonts: &mut FontLibrary, builders: &mut LayoutContext<()>) -> parley::Layout<()> {
        let size = f32::from_bits(key.size_bits);
        let features = key
            .features
            .iter()
            .map(|tag| format!("\"{tag}\" on"))
            .collect::<Vec<_>>()
            .join(", ");
        let variations = format!("\"{OPTICAL_SIZE_AXIS}\" {size}");
        let mut builder = builders.ranged_builder(
            fonts.context_mut(),
            &key.content,
            f32::from_bits(key.scale_factor_bits),
            true,
        );
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(&key.family)));
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(f32::from_bits(
            key.weight_bits,
        ))));
        builder.push_default(StyleProperty::LetterSpacing(
            f32::from_bits(key.letter_spacing_bits) * size,
        ));
        builder.push_default(StyleProperty::FontFeatures(features.as_str().into()));
        builder.push_default(StyleProperty::FontVariations(variations.as_str().into()));

        builder.build(&key.content)
    }
}
