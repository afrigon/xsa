use anyhow::Context;
use parley::PositionedLayoutItem;

use crate::atlas::{GlyphKey, SUBPIXEL_STEPS};
use crate::shaping::{ShapedText, TextInputs};
use crate::{
    DrawList, GlyphPrimitive, LayoutContext, LinearColor, Node, NodeLayout, Point, Primitive, Rect, Size, SizeProposal,
};

// A Text's shaped text and the inputs it was shaped from; without a font there is nothing to lay out.
pub(crate) struct TextLayout {
    pub inputs: Option<TextInputs>,
    pub shaped: Option<ShapedText>,
    pub color: LinearColor,
}

impl TextLayout {
    pub fn new() -> TextLayout {
        TextLayout {
            inputs: None,
            shaped: None,
            color: LinearColor::WHITE,
        }
    }
}

impl NodeLayout for TextLayout {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        _children: &mut [Node],
        _context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        let (Some(inputs), Some(shaped)) = (&self.inputs, &mut self.shaped) else {
            return Ok(Size::default());
        };
        let scale_factor = inputs.scale_factor;
        let size = shaped.size(proposal.width.map(|width| width * scale_factor));

        Ok(Size {
            width: size.width / scale_factor,
            height: size.height / scale_factor,
        })
    }

    fn place(
        &mut self,
        bounds: Rect,
        _children: &mut [Node],
        context: &mut LayoutContext,
        draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        let (Some(inputs), Some(shaped)) = (&self.inputs, &mut self.shaped) else {
            return Ok(());
        };
        let scale_factor = inputs.scale_factor;
        let layout = shaped.broken_at(Some(bounds.size.width * scale_factor));
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
                        color: self.color,
                    }));
                }
            }
        }

        Ok(())
    }
}
