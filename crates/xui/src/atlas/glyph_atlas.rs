use std::collections::HashMap;

use anyhow::{Context, ensure};
use parley::FontData;
use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::{Format, Vector};

use super::{AtlasGlyph, AtlasRegion, AtlasUpdate, GlyphKey};

pub const ATLAS_SIZE: u32 = 1024;
pub(crate) const SUBPIXEL_STEPS: u8 = 4;
const GLYPH_PADDING: u32 = 1;

pub(crate) struct GlyphAtlas {
    glyphs: HashMap<GlyphKey, Option<AtlasGlyph>>,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    updates: Vec<AtlasUpdate>,
    scale: ScaleContext,
}

impl GlyphAtlas {
    pub fn new() -> GlyphAtlas {
        GlyphAtlas {
            glyphs: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            updates: Vec::new(),
            scale: ScaleContext::new(),
        }
    }

    pub fn glyph(&mut self, key: GlyphKey, font: &FontData, size: f32) -> anyhow::Result<Option<AtlasGlyph>> {
        if let Some(glyph) = self.glyphs.get(&key) {
            return Ok(*glyph);
        }

        let font_reference =
            FontRef::from_index(font.data.data(), font.index as usize).context("the font data is not a font")?;
        let mut scaler = self
            .scale
            .builder(font_reference)
            .size(size)
            .hint(true)
            .normalized_coords(&key.coordinates)
            .build();
        let subpixel_offset = f32::from(key.subpixel_step) / f32::from(SUBPIXEL_STEPS);
        let image = Render::new(&[Source::Outline])
            .format(Format::Alpha)
            .offset(Vector::new(subpixel_offset, 0.0))
            .render(&mut scaler, key.glyph_id);
        let glyph = match image {
            Some(image) if image.placement.width > 0 && image.placement.height > 0 => {
                let region = self.allocate(image.placement.width, image.placement.height)?;
                self.updates.push(AtlasUpdate {
                    region,
                    pixels: image.data,
                });

                Some(AtlasGlyph {
                    region,
                    left: image.placement.left,
                    top: image.placement.top,
                })
            }
            _ => None,
        };

        self.glyphs.insert(key, glyph);

        Ok(glyph)
    }

    pub fn take_updates(&mut self) -> Vec<AtlasUpdate> {
        std::mem::take(&mut self.updates)
    }

    fn allocate(&mut self, width: u32, height: u32) -> anyhow::Result<AtlasRegion> {
        if self.cursor_x + width > ATLAS_SIZE {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + GLYPH_PADDING;
            self.row_height = 0;
        }

        ensure!(
            width <= ATLAS_SIZE && self.cursor_y + height <= ATLAS_SIZE,
            "the glyph atlas is full"
        );
        let region = AtlasRegion {
            x: self.cursor_x,
            y: self.cursor_y,
            width,
            height,
        };
        self.cursor_x += width + GLYPH_PADDING;
        self.row_height = self.row_height.max(height);

        Ok(region)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_regions_in_rows_without_overlap() {
        let mut atlas = GlyphAtlas::new();
        let first = atlas.allocate(600, 20).unwrap();
        let second = atlas.allocate(600, 30).unwrap();
        let third = atlas.allocate(10, 10).unwrap();

        assert_eq!((first.x, first.y), (0, 0));
        assert_eq!((second.x, second.y), (0, 20 + GLYPH_PADDING));
        assert_eq!((third.x, third.y), (600 + GLYPH_PADDING, 20 + GLYPH_PADDING));
    }

    #[test]
    fn fails_when_full() {
        let mut atlas = GlyphAtlas::new();

        for _ in 0..ATLAS_SIZE {
            if atlas.allocate(ATLAS_SIZE, 1).is_err() {
                return;
            }
        }

        panic!("the atlas never filled up");
    }
}
