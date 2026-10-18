use std::io::Cursor;

use glam::DVec3;
use resource_loader::terrain::{CubeFace, PAGE_BORDER, PAGE_EXTENT, PAGE_INTERIOR, PageEncoding, TerrainReader};
use resource_loader::texture::{Ktx2Loader, TextureFormat, TextureLoader};

use super::{CubeMapInput, LayerEncoding, TerrainWriter};

const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
const KTX2_HEADER_SIZE: usize = 80;
const KTX2_LEVEL_ENTRY_SIZE: usize = 24;
const FACE_EXTENT: u32 = 256;
const HEIGHT_GRADIENT: DVec3 = DVec3::new(1000.0, -2000.0, 3000.0);

// A KTX2 cube map whose texels come from a function of the face, mip level and texel.
fn cube_map(
    format: TextureFormat,
    extent: u32,
    levels: u32,
    texel: impl Fn(CubeFace, u32, u32, u32) -> Vec<u8>,
) -> Vec<u8> {
    let data: Vec<Vec<u8>> = (0..levels)
        .map(|mip| {
            let extent = (extent >> mip).max(1);
            CubeFace::ALL
                .iter()
                .flat_map(|&face| {
                    (0..extent * extent)
                        .flat_map(|index| texel(face, extent, index % extent, index / extent))
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .collect();
    let mut bytes = KTX2_IDENTIFIER.to_vec();

    for word in [format.vulkan_format(), 1, extent, extent, 0, 0, 6, levels, 0] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }

    bytes.resize(KTX2_HEADER_SIZE, 0);
    let mut offset = KTX2_HEADER_SIZE + data.len() * KTX2_LEVEL_ENTRY_SIZE;

    for level in &data {
        for long in [offset, level.len(), level.len()] {
            bytes.extend_from_slice(&(long as u64).to_le_bytes());
        }
        offset += level.len();
    }

    for level in &data {
        bytes.extend_from_slice(level);
    }

    bytes
}

fn input(bytes: &[u8]) -> anyhow::Result<CubeMapInput> {
    CubeMapInput::new(Ktx2Loader::load(bytes)?)
}

// The direction from the body's centre through a texel's centre, `x` and `y` possibly beyond the face.
fn direction(face: CubeFace, extent: u32, x: f64, y: f64) -> DVec3 {
    let along = |coordinate: f64| (2.0 * coordinate + 1.0) / f64::from(extent) - 1.0;

    (face.normal().as_dvec3() + face.column_axis().as_dvec3() * along(x) + face.row_axis().as_dvec3() * along(y))
        .normalize()
}

fn height(direction: DVec3) -> f32 {
    direction.dot(HEIGHT_GRADIENT) as f32
}

fn heights() -> Vec<u8> {
    let levels = FACE_EXTENT.ilog2() + 1;

    cube_map(TextureFormat::R32Float, FACE_EXTENT, levels, |face, extent, x, y| {
        height(direction(face, extent, f64::from(x), f64::from(y)))
            .to_le_bytes()
            .to_vec()
    })
}

fn converted(bytes: &[u8], encoding: PageEncoding) -> TerrainReader<Cursor<Vec<u8>>> {
    let input = input(bytes).unwrap();
    let encoding = LayerEncoding::new(encoding, input.format()).unwrap();
    let mut file = Vec::new();
    TerrainWriter::new(&input, encoding).write(&mut file).unwrap();

    TerrainReader::new(Cursor::new(file)).unwrap()
}

#[test]
fn heights_read_back_with_their_borders_and_ranges() {
    let mut reader = converted(&heights(), PageEncoding::PageHeights);
    let header = *reader.header();

    assert_eq!(header.format, TextureFormat::R16Unorm);
    assert_eq!(header.encoding, PageEncoding::PageHeights);
    assert_eq!(header.level_count, 2);

    for face in CubeFace::ALL {
        for level in 0..header.level_count {
            let extent = PAGE_INTERIOR << level;
            let entries = reader.entries(face, level).unwrap();
            assert_eq!(entries.len(), 1 << (2 * level));
            assert!(entries.is_sorted_by_key(|entry| entry.key));

            for entry in entries {
                let range = entry.range.unwrap();
                let page = reader.page(&entry).unwrap();
                assert_eq!(page.len(), (PAGE_EXTENT * PAGE_EXTENT) as usize * size_of::<u16>());

                let values: Vec<u16> = page
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|value| u16::from_le_bytes(*value))
                    .collect();
                assert_eq!(values.iter().min(), Some(&0));
                assert_eq!(values.iter().max(), Some(&u16::MAX));

                let step = (range.maximum - range.minimum) / f32::from(u16::MAX);
                // Border texels across a face edge or at a corner come from texels a few texels away on the cube.
                let edge_tolerance = (HEIGHT_GRADIENT.length() * 16.0 / f64::from(extent)) as f32;

                for (index, value) in values.iter().enumerate() {
                    let column = (index as u32 % PAGE_EXTENT) as i32 - PAGE_BORDER as i32;
                    let row = (index as u32 / PAGE_EXTENT) as i32 - PAGE_BORDER as i32;
                    let x = (entry.key.x() * PAGE_INTERIOR) as i32 + column;
                    let y = (entry.key.y() * PAGE_INTERIOR) as i32 + row;
                    let expected = height(direction(face, extent, f64::from(x), f64::from(y)));
                    let inside = |coordinate: i32| (0..extent as i32).contains(&coordinate);
                    let tolerance = if inside(x) && inside(y) { step } else { edge_tolerance };

                    assert!(
                        (range.height(*value) - expected).abs() <= tolerance,
                        "{face:?} level {level} page {:?} texel ({column}, {row}): {} instead of {expected}",
                        entry.key,
                        range.height(*value)
                    );
                }
            }
        }
    }
}

#[test]
fn colours_normals_and_hapke_maps_keep_their_texel_formats() {
    let levels = FACE_EXTENT.ilog2() + 1;
    let cases = [
        (
            TextureFormat::Rgba8Srgb,
            TextureFormat::Bc7Srgb,
            vec![200, 100, 50, 255],
        ),
        (TextureFormat::Rg8Unorm, TextureFormat::Bc5Unorm, vec![128, 64]),
        (
            TextureFormat::Rgba16Float,
            TextureFormat::Rgba16Float,
            vec![0, 60, 0, 56, 0, 52, 0, 60],
        ),
    ];

    for (input_format, page_format, texel) in cases {
        let bytes = cube_map(input_format, FACE_EXTENT, levels, |_, _, _, _| texel.clone());
        let mut reader = converted(&bytes, PageEncoding::Direct);
        assert_eq!(reader.header().format, page_format);
        assert_eq!(reader.header().encoding, PageEncoding::Direct);

        let entry = reader.entries(CubeFace::NegativeZ, 1).unwrap()[3];
        assert!(entry.range.is_none());
        assert_eq!(
            reader.page(&entry).unwrap().len(),
            page_format.level_size(PAGE_EXTENT, PAGE_EXTENT)
        );
    }
}

#[test]
fn inputs_matching_no_layer_are_refused() {
    let levels = FACE_EXTENT.ilog2() + 1;
    let refused = |format: TextureFormat, encoding: PageEncoding| {
        let bytes = cube_map(format, FACE_EXTENT, levels, |_, _, _, _| vec![0; format.block_size()]);
        LayerEncoding::new(encoding, input(&bytes).unwrap().format()).is_err()
    };

    assert!(refused(TextureFormat::Rgba8Unorm, PageEncoding::Direct));
    assert!(refused(TextureFormat::R16Unorm, PageEncoding::PageHeights));
    assert!(refused(TextureFormat::R32Float, PageEncoding::Direct));
    assert!(refused(TextureFormat::Rgba8Srgb, PageEncoding::PageHeights));
}

#[test]
fn cube_maps_the_conversion_cannot_page_are_refused() {
    let texel = |_: CubeFace, _: u32, _: u32, _: u32| 0f32.to_le_bytes().to_vec();
    let refusal = |bytes: Vec<u8>| input(&bytes).err().unwrap().to_string();

    assert!(refusal(cube_map(TextureFormat::R32Float, 64, 7, texel)).contains("at least 128"));
    assert!(refusal(cube_map(TextureFormat::R32Float, 192, 8, texel)).contains("power of two"));
    assert!(refusal(cube_map(TextureFormat::R32Float, 256, 3, texel)).contains("full chain of 9"));
    // One byte per texel is one 16-byte block per 4×4 texels, down to the 4×4 mip level.
    assert!(refusal(cube_map(TextureFormat::Bc7Srgb, 256, 7, |_, _, _, _| vec![0])).contains("uncompressed"));
}

#[test]
fn colours_average_in_linear_light() {
    let mut output = [0; 4];
    LayerEncoding::Bc7.average(&[&[0, 0, 0, 0], &[255, 255, 255, 255], &[0, 0, 0, 255]], &mut output);

    assert_eq!(output, [156, 156, 156, 170]);
}
