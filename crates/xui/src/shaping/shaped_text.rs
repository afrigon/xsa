use std::collections::HashMap;

use parley::{AlignmentOptions, FontFamily, FontWeight, Layout, StyleProperty};

use super::TextInputs;
use crate::{FontLibrary, Size};

const OPTICAL_SIZE_AXIS: &str = "opsz";

// A shaped text, broken into lines for one width at a time; sizes measured at other widths are remembered.
// Widths and sizes are in physical pixels.
pub(crate) struct ShapedText {
    layout: Layout<()>,
    broken_at: Option<Option<u32>>,
    sizes: HashMap<Option<u32>, Size>,
}

impl ShapedText {
    // Parley scales every metric by the scale factor, so the result is in physical pixels.
    pub fn shape(inputs: &TextInputs, fonts: &mut FontLibrary, builders: &mut parley::LayoutContext<()>) -> ShapedText {
        let font = &inputs.font;
        let features = font
            .features
            .iter()
            .map(|tag| format!("\"{tag}\" on"))
            .collect::<Vec<_>>()
            .join(", ");
        let variations = format!("\"{OPTICAL_SIZE_AXIS}\" {}", font.size);
        let mut builder = builders.ranged_builder(fonts.context_mut(), &inputs.content, inputs.scale_factor, true);
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(&font.family)));
        builder.push_default(StyleProperty::FontSize(font.size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(font.weight)));
        builder.push_default(StyleProperty::LetterSpacing(font.letter_spacing * font.size));
        builder.push_default(StyleProperty::FontFeatures(features.as_str().into()));
        builder.push_default(StyleProperty::FontVariations(variations.as_str().into()));

        ShapedText {
            layout: builder.build(&inputs.content),
            broken_at: None,
            sizes: HashMap::new(),
        }
    }

    pub fn size(&mut self, max_width: Option<f32>) -> Size {
        let width_bits = max_width.map(f32::to_bits);

        if let Some(size) = self.sizes.get(&width_bits) {
            return *size;
        }

        let size = {
            let layout = self.broken_at(max_width);
            Size {
                width: layout.width(),
                height: layout.height(),
            }
        };
        self.sizes.insert(width_bits, size);

        size
    }

    pub fn broken_at(&mut self, max_width: Option<f32>) -> &Layout<()> {
        let width_bits = max_width.map(f32::to_bits);

        if self.broken_at != Some(width_bits) {
            self.layout.break_all_lines(max_width);
            self.layout.align(parley::Alignment::Start, AlignmentOptions::default());
            self.broken_at = Some(width_bits);
        }

        &self.layout
    }
}
