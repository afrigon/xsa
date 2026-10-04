use anyhow::Context;
use parley::{AlignmentOptions, FontFamily, FontWeight, Layout, PositionedLayoutItem, StyleProperty};

use crate::atlas::{GlyphKey, SUBPIXEL_STEPS};
use crate::{
    DrawList, Environment, FontKey, ForegroundStyleKey, GlyphPrimitive, Never, Point, Primitive, Rect, ScaleFactorKey,
    Size, SizeProposal, View, ViewContext,
};

const OPTICAL_SIZE_AXIS: &str = "opsz";

// Draws in the environment's font and foreground style; without a font it takes no space.
pub struct Text {
    content: String,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Text {
        Text {
            content: content.into(),
        }
    }

    // Laid out in physical pixels: parley scales every metric by the environment's scale factor.
    fn layout(&self, context: &mut ViewContext, max_width: Option<f32>) -> Option<Layout<()>> {
        let Some(font) = context.environment.get::<FontKey>() else {
            context.warn_once(format!("no font in the environment for the text {:?}", self.content));
            return None;
        };
        let scale_factor = context.environment.get::<ScaleFactorKey>();
        let features = font
            .features
            .iter()
            .map(|tag| format!("\"{tag}\" on"))
            .collect::<Vec<_>>()
            .join(", ");
        let variations = format!("\"{OPTICAL_SIZE_AXIS}\" {}", font.size);
        let mut builder =
            context
                .layouts
                .ranged_builder(context.fonts.context_mut(), &self.content, scale_factor, true);
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(&font.family)));
        builder.push_default(StyleProperty::FontSize(font.size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(font.weight)));
        builder.push_default(StyleProperty::LetterSpacing(font.letter_spacing * font.size));
        builder.push_default(StyleProperty::FontFeatures(features.as_str().into()));
        builder.push_default(StyleProperty::FontVariations(variations.as_str().into()));
        let mut layout = builder.build(&self.content);
        layout.break_all_lines(max_width.map(|width| width * scale_factor));
        layout.align(parley::Alignment::Start, AlignmentOptions::default());

        Some(layout)
    }
}

impl View for Text {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let Some(layout) = self.layout(context, proposal.width) else {
            return Ok(Size::default());
        };
        let scale_factor = context.environment.get::<ScaleFactorKey>();

        Ok(Size {
            width: layout.width() / scale_factor,
            height: layout.height() / scale_factor,
        })
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        let Some(layout) = self.layout(context, Some(bounds.size.width)) else {
            return Ok(());
        };
        let scale_factor = context.environment.get::<ScaleFactorKey>();
        let color = context.environment.get::<ForegroundStyleKey>();
        let origin = Point {
            x: bounds.origin.x * scale_factor,
            y: bounds.origin.y * scale_factor,
        };

        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let font = run.font();

                for glyph in glyph_run.positioned_glyphs() {
                    let x = origin.x + glyph.x;
                    let whole_x = x.floor();
                    let subpixel_step = ((x - whole_x) * f32::from(SUBPIXEL_STEPS)) as u8;
                    let key = GlyphKey {
                        font_id: font.data.id(),
                        font_index: font.index,
                        glyph_id: u16::try_from(glyph.id).context("glyph ids above 65535 are not supported")?,
                        size_bits: run.font_size().to_bits(),
                        coordinates: run.normalized_coords().to_vec(),
                        subpixel_step,
                    };
                    let Some(atlas_glyph) = context.atlas.glyph(key, font, run.font_size())? else {
                        continue;
                    };

                    draw_list.primitives.push(Primitive::Glyph(GlyphPrimitive {
                        bounds: Rect {
                            origin: Point {
                                x: whole_x + atlas_glyph.left as f32,
                                y: (origin.y + glyph.y).round() - atlas_glyph.top as f32,
                            },
                            size: Size {
                                width: atlas_glyph.region.width as f32,
                                height: atlas_glyph.region.height as f32,
                            },
                        },
                        atlas_region: atlas_glyph.region,
                        color,
                    }));
                }
            }
        }

        Ok(())
    }
}
