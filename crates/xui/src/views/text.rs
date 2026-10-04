use anyhow::Context;
use parley::PositionedLayoutItem;

use crate::atlas::{GlyphKey, SUBPIXEL_STEPS};
use crate::text_layouts::{CachedTextLayout, TextLayoutKey};
use crate::{
    DrawList, Environment, FontKey, ForegroundStyleKey, GlyphPrimitive, Never, Point, Primitive, Rect, ScaleFactorKey,
    Size, SizeProposal, View, ViewContext,
};

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

    // The text's shaped layout, in physical pixels; without a font in the environment there is none.
    fn layout<'a>(&self, context: &'a mut ViewContext) -> Option<&'a mut CachedTextLayout> {
        let Some(font) = context.environment.get::<FontKey>() else {
            context.warn_once(format!("no font in the environment for the text {:?}", self.content));
            return None;
        };
        let key = TextLayoutKey::new(&self.content, &font, context.environment.get::<ScaleFactorKey>());

        Some(context.text_layouts.get(key, context.fonts, context.layouts))
    }
}

impl View for Text {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let scale_factor = context.environment.get::<ScaleFactorKey>();
        let Some(layout) = self.layout(context) else {
            return Ok(Size::default());
        };
        let size = layout.size(proposal.width.map(|width| width * scale_factor));

        Ok(Size {
            width: size.width / scale_factor,
            height: size.height / scale_factor,
        })
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        let scale_factor = context.environment.get::<ScaleFactorKey>();
        let color = context.environment.get::<ForegroundStyleKey>();
        let Some(font) = context.environment.get::<FontKey>() else {
            context.warn_once(format!("no font in the environment for the text {:?}", self.content));
            return Ok(());
        };
        let key = TextLayoutKey::new(&self.content, &font, scale_factor);
        let layout = context
            .text_layouts
            .get(key, context.fonts, context.layouts)
            .broken_at(Some(bounds.size.width * scale_factor));
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
