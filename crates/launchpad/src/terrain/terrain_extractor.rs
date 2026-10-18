use std::io::{Read, Seek};

use anyhow::{Context, bail, ensure};
use resource_loader::terrain::{CubeFace, PAGE_BORDER, PAGE_EXTENT, PAGE_INTERIOR, TerrainReader};
use resource_loader::texture::TextureImage;

use super::{LayerEncoding, LevelSelection, TerrainRegion};

/// Decodes the pages of a `.terrain` file back into the texels of the cube map it was made from.
pub struct TerrainExtractor<R> {
    reader: TerrainReader<R>,
    layer: LayerEncoding,
}

impl<R: Read + Seek> TerrainExtractor<R> {
    pub fn new(reader: TerrainReader<R>) -> anyhow::Result<TerrainExtractor<R>> {
        let header = reader.header();
        let layer = LayerEncoding::from_page_format(header.format, header.encoding)?;

        Ok(TerrainExtractor { reader, layer })
    }

    pub fn layer(&self) -> LayerEncoding {
        self.layer
    }

    pub fn extract(&mut self, region: TerrainRegion) -> anyhow::Result<TextureImage> {
        match region {
            TerrainRegion::Cube(levels) => self.faces(&CubeFace::ALL, levels),
            TerrainRegion::Face(face, levels) => self.faces(&[face], levels),
            TerrainRegion::Page(address) => {
                let entry = self
                    .reader
                    .entry(address)?
                    .with_context(|| format!("{region}: the file holds no such page"))?;
                let texels = self.layer.decode(&self.reader.page(&entry)?, entry.range);

                TextureImage::new(self.layer.input_format(), PAGE_EXTENT, PAGE_EXTENT, 1, vec![texels])
            }
        }
    }

    // Each face's mip chain over the selected levels, finest first.
    fn faces(&mut self, faces: &[CubeFace], levels: LevelSelection) -> anyhow::Result<TextureImage> {
        let level_count = self.reader.header().level_count;
        ensure!(level_count > 0, "the file holds no levels");

        let levels: Vec<u32> = match levels {
            LevelSelection::Every => (0..level_count).rev().collect(),
            LevelSelection::One(level) => {
                ensure!(level < level_count, "level {level}: the file has {level_count} levels");
                vec![level]
            }
        };
        let mut layers = Vec::new();

        for &face in faces {
            let mut layer = Vec::new();

            for &level in &levels {
                layer.extend(self.face(face, level)?);
            }

            layers.push(layer);
        }

        let extent = PAGE_INTERIOR << levels[0];
        TextureImage::new(self.layer.input_format(), extent, extent, levels.len() as u32, layers)
    }

    // A face at a level, stitched from its pages' interiors, absent pages filled with the absent texel.
    fn face(&mut self, face: CubeFace, level: u32) -> anyhow::Result<Vec<u8>> {
        let texel_bytes = self.layer.input_format().block_size();
        let extent = (PAGE_INTERIOR << level) as usize;
        let pages_per_side = 1u64 << level;
        let interior = PAGE_INTERIOR as usize;
        let mut texels = self.layer.absent_texel().repeat(extent * extent);

        for entry in self.reader.entries(face, level)? {
            let (x, y) = (entry.key.x() as usize, entry.key.y() as usize);

            if x as u64 >= pages_per_side || y as u64 >= pages_per_side {
                bail!("{face} level {level} page {x},{y} is outside its level: the file is broken");
            }

            let page = self.layer.decode(&self.reader.page(&entry)?, entry.range);

            for row in 0..interior {
                let source = ((row + PAGE_BORDER as usize) * PAGE_EXTENT as usize + PAGE_BORDER as usize) * texel_bytes;
                let target = ((y * interior + row) * extent + x * interior) * texel_bytes;
                texels[target..target + interior * texel_bytes]
                    .copy_from_slice(&page[source..source + interior * texel_bytes]);
            }
        }

        Ok(texels)
    }
}
