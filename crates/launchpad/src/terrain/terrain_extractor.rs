use std::io::{Read, Seek};

use anyhow::{Context, bail, ensure};
use resource_loader::terrain::{PAGE_BORDER, PAGE_EXTENT, PAGE_INTERIOR, TerrainReader};
use resource_loader::texture::{TextureImage, TextureKind};
use resource_loader::utils::CubeFace;

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
            TerrainRegion::Cube(levels) => self.faces(&CubeFace::ALL, TextureKind::CubeMap, levels),
            TerrainRegion::Face(face, levels) => self.faces(&[face], TextureKind::Plane, levels),
            TerrainRegion::Page(address) => {
                let entry = self
                    .reader
                    .entry(address)?
                    .with_context(|| format!("{region}: the file holds no such page"))?;
                let texels = self.layer.decode(&self.reader.page(&entry)?, entry.range);

                TextureImage::new(
                    self.layer.input_format(),
                    TextureKind::Plane,
                    PAGE_EXTENT,
                    PAGE_EXTENT,
                    1,
                    vec![texels],
                )
            }
        }
    }

    // Each face's mip chain over the selected levels, finest first.
    fn faces(&mut self, faces: &[CubeFace], kind: TextureKind, levels: LevelSelection) -> anyhow::Result<TextureImage> {
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
        TextureImage::new(
            self.layer.input_format(),
            kind,
            extent,
            extent,
            levels.len() as u32,
            layers,
        )
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use resource_loader::terrain::{HeightRange, LevelRecord, PageAddress, PageEncoding, PageKey, TerrainHeader};
    use resource_loader::texture::TextureFormat;
    use resource_loader::utils::HalfFloat;

    use super::*;
    use crate::terrain::{CubeMapInput, TerrainWriter, TestCubeMap};

    const FACE_EXTENT: u32 = 256;
    const HALF_TEXEL_BYTES: usize = 8;

    fn half_texel(face: CubeFace, extent: u32, x: u32, y: u32) -> Vec<u8> {
        [face.index() as f32, x as f32, y as f32, extent as f32]
            .iter()
            .flat_map(|&value| HalfFloat::from_f32(value).bits.to_le_bytes())
            .collect()
    }

    fn height(face: CubeFace, extent: u32, x: u32, y: u32) -> f32 {
        face.index() as f32 * 1000.0 + x as f32 * 3.25 - y as f32 * 1.5 + extent as f32
    }

    fn converted(input: &CubeMapInput, encoding: PageEncoding) -> Vec<u8> {
        let encoding = LayerEncoding::new(encoding, input.format()).unwrap();
        let mut file = Vec::new();
        TerrainWriter::new(input, encoding).write(&mut file).unwrap();

        file
    }

    fn extracted(file: Vec<u8>, region: TerrainRegion) -> TextureImage {
        TerrainExtractor::new(TerrainReader::new(Cursor::new(file)).unwrap())
            .unwrap()
            .extract(region)
            .unwrap()
    }

    fn half_input() -> (Vec<u8>, Vec<Vec<u8>>) {
        let cube_map = TestCubeMap::full_chain(TextureFormat::Rgba16Float, FACE_EXTENT);
        let input = cube_map.input(half_texel).unwrap();
        let file = converted(&input, PageEncoding::Direct);
        let layers = CubeFace::ALL
            .iter()
            .map(|&face| {
                [FACE_EXTENT, FACE_EXTENT / 2]
                    .iter()
                    .flat_map(|&extent| {
                        (0..extent * extent).flat_map(move |i| half_texel(face, extent, i % extent, i / extent))
                    })
                    .collect()
            })
            .collect();

        (file, layers)
    }

    #[test]
    fn a_cube_map_extracts_exactly_down_to_its_coarsest_level() {
        let (file, layers) = half_input();
        let image = extracted(file, TerrainRegion::Cube(LevelSelection::Every));

        assert_eq!(image.format(), TextureFormat::Rgba16Float);
        assert_eq!((image.width(), image.mip_levels()), (FACE_EXTENT, 2));
        assert_eq!(image.layers(), layers.as_slice());
    }

    #[test]
    fn heights_extract_within_half_a_step_of_their_page_range() {
        let input = TestCubeMap::full_chain(TextureFormat::R32Float, FACE_EXTENT)
            .input(|face, extent, x, y| height(face, extent, x, y).to_le_bytes().to_vec())
            .unwrap();
        let file = converted(&input, PageEncoding::PageHeights);
        let mut reader = TerrainReader::new(Cursor::new(file.clone())).unwrap();
        let ranges: Vec<HeightRange> = reader
            .entries(CubeFace::NegativeY, 1)
            .unwrap()
            .iter()
            .map(|entry| entry.range.unwrap())
            .collect();
        let image = extracted(file, TerrainRegion::Face(CubeFace::NegativeY, LevelSelection::One(1)));

        assert_eq!(
            (image.format(), image.width(), image.mip_levels()),
            (TextureFormat::R32Float, FACE_EXTENT, 1)
        );

        for (index, bytes) in image.layers()[0].as_chunks::<4>().0.iter().enumerate() {
            let (x, y) = (index as u32 % FACE_EXTENT, index as u32 / FACE_EXTENT);
            let range = ranges[PageKey::from_coordinates(x / PAGE_INTERIOR, y / PAGE_INTERIOR).code as usize];
            let half_step = (range.maximum - range.minimum) / f32::from(u16::MAX) / 2.0;
            let expected = height(CubeFace::NegativeY, FACE_EXTENT, x, y);

            assert!(
                (f32::from_le_bytes(*bytes) - expected).abs() <= half_step + 1e-3,
                "{x},{y}"
            );
        }
    }

    #[test]
    fn a_page_extracts_with_its_border() {
        let (file, layers) = half_input();
        let address = PageAddress {
            face: CubeFace::PositiveX,
            level: 1,
            key: PageKey::from_coordinates(1, 0),
        };
        let image = extracted(file, TerrainRegion::Page(address));
        let page = &image.layers()[0];
        let face = &layers[CubeFace::PositiveX.index()];
        let texel = |data: &[u8], width: u32, x: u32, y: u32| {
            let offset = (y * width + x) as usize * HALF_TEXEL_BYTES;
            data[offset..offset + HALF_TEXEL_BYTES].to_vec()
        };

        assert_eq!((image.width(), image.height()), (PAGE_EXTENT, PAGE_EXTENT));
        assert_eq!(
            texel(page, PAGE_EXTENT, 0, PAGE_BORDER),
            texel(face, FACE_EXTENT, PAGE_INTERIOR - PAGE_BORDER, 0)
        );
        assert_eq!(
            texel(page, PAGE_EXTENT, PAGE_BORDER, PAGE_BORDER),
            texel(face, FACE_EXTENT, PAGE_INTERIOR, 0)
        );
    }

    #[test]
    fn an_absent_page_extracts_transparent() {
        let (mut file, layers) = half_input();
        let level_count = 2;
        let record = TerrainHeader::SIZE + (CubeFace::PositiveX.index() * level_count + 1) * LevelRecord::SIZE;
        let entry_count = record + size_of::<u64>();
        file[entry_count..entry_count + size_of::<u64>()].copy_from_slice(&3u64.to_le_bytes());
        let image = extracted(file, TerrainRegion::Face(CubeFace::PositiveX, LevelSelection::One(1)));
        let row_bytes = FACE_EXTENT as usize * HALF_TEXEL_BYTES;
        let half = row_bytes / 2;

        for (y, (row, expected)) in image.layers()[0]
            .chunks(row_bytes)
            .zip(layers[0].chunks(row_bytes))
            .enumerate()
        {
            if y < PAGE_INTERIOR as usize {
                assert_eq!(row, expected);
            } else {
                assert_eq!(&row[..half], &expected[..half]);
                assert!(row[half..].iter().all(|&byte| byte == 0));
            }
        }
    }
}
