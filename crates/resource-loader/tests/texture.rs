use resource_loader::texture::{
    ColorSpace, DdsLoader, Ktx2Loader, Ktx2Writer, PngLoader, TextureFormat, TextureImage, TextureLoader,
};

const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
const KTX2_HEADER_SIZE: usize = 80;
const KTX2_LEVEL_ENTRY_SIZE: usize = 24;
const VK_FORMAT_R8G8B8A8_UNORM: u32 = 37;
const VK_FORMAT_R8G8B8A8_SRGB: u32 = 43;
const VK_FORMAT_R16_UNORM: u32 = 70;
const VK_FORMAT_R32_SFLOAT: u32 = 100;
const VK_FORMAT_BC4_UNORM_BLOCK: u32 = 139;
const SUPERCOMPRESSION_NONE: u32 = 0;
const SUPERCOMPRESSION_ZSTANDARD: u32 = 2;
const DDS_HEADER_SIZE: usize = 148;
const DXGI_FORMAT_BC7_UNORM: u32 = 98;
const BC7_BLOCK_BYTES: usize = 16;

struct Ktx2 {
    format: u32,
    width: u32,
    height: u32,
    faces: u32,
    supercompression: u32,
    // Each level's data, all faces in order.
    levels: Vec<Vec<u8>>,
}

impl Ktx2 {
    fn bytes(&self) -> Vec<u8> {
        let stored: Vec<Vec<u8>> = self
            .levels
            .iter()
            .map(|level| match self.supercompression {
                SUPERCOMPRESSION_ZSTANDARD => zstd::bulk::compress(level, 3).unwrap(),
                _ => level.clone(),
            })
            .collect();
        let mut bytes = KTX2_IDENTIFIER.to_vec();

        for word in [
            self.format,
            1,
            self.width,
            self.height,
            0,
            0,
            self.faces,
            self.levels.len() as u32,
            self.supercompression,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }

        bytes.resize(KTX2_HEADER_SIZE, 0);
        let mut offset = KTX2_HEADER_SIZE + self.levels.len() * KTX2_LEVEL_ENTRY_SIZE;

        for (level, data) in self.levels.iter().zip(&stored) {
            for long in [offset, data.len(), level.len()] {
                bytes.extend_from_slice(&(long as u64).to_le_bytes());
            }
            offset += data.len();
        }

        for data in &stored {
            bytes.extend_from_slice(data);
        }

        bytes
    }
}

// A DDS header with a DX10 extension, for a BC7 texture.
fn dds_header(width: u32, height: u32, mip_levels: u32) -> Vec<u8> {
    let mut bytes = vec![0; DDS_HEADER_SIZE];
    bytes[..4].copy_from_slice(b"DDS ");
    bytes[12..16].copy_from_slice(&height.to_le_bytes());
    bytes[16..20].copy_from_slice(&width.to_le_bytes());
    bytes[28..32].copy_from_slice(&mip_levels.to_le_bytes());
    bytes[84..88].copy_from_slice(b"DX10");
    bytes[128..132].copy_from_slice(&DXGI_FORMAT_BC7_UNORM.to_le_bytes());
    bytes
}

// Every byte of a level's face holds the level and face it belongs to.
fn tagged(level: u8, face: u8, size: usize) -> Vec<u8> {
    vec![level * 16 + face; size]
}

#[test]
fn a_ktx2_cube_map_regroups_its_levels_by_face() {
    let levels = [(2, 2), (1, 1)]
        .iter()
        .enumerate()
        .map(|(level, (width, height))| {
            (0..6)
                .flat_map(|face| tagged(level as u8, face, width * height * 4))
                .collect()
        })
        .collect();
    let file = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_UNORM,
        width: 2,
        height: 2,
        faces: 6,
        supercompression: SUPERCOMPRESSION_ZSTANDARD,
        levels,
    };
    let image = Ktx2Loader::load(&file.bytes())
        .unwrap()
        .in_color_space(ColorSpace::Srgb)
        .unwrap();

    assert_eq!(image.format(), TextureFormat::Rgba8Srgb);
    assert_eq!(image.mip_levels(), 2);
    assert_eq!(image.layers().len(), 6);

    for (face, layer) in image.layers().iter().enumerate() {
        let mut expected = tagged(0, face as u8, 16);
        expected.extend(tagged(1, face as u8, 4));
        assert_eq!(layer, &expected);
    }
}

#[test]
fn an_uncompressed_ktx2_height_map_reads_as_is() {
    let file = Ktx2 {
        format: VK_FORMAT_R16_UNORM,
        width: 4,
        height: 2,
        faces: 1,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![(0..16).collect()],
    };
    let image = Ktx2Loader::load(&file.bytes())
        .unwrap()
        .in_color_space(ColorSpace::Linear)
        .unwrap();

    assert_eq!(image.format(), TextureFormat::R16Unorm);
    assert_eq!((image.width(), image.height()), (4, 2));
    assert_eq!(image.layers(), &[(0..16).collect::<Vec<u8>>()]);
}

#[test]
fn a_ktx2_texture_keeps_the_format_it_stores() {
    let colors = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_SRGB,
        width: 1,
        height: 1,
        faces: 1,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![vec![0; 4]],
    };
    let heights = Ktx2 {
        format: VK_FORMAT_R32_SFLOAT,
        levels: vec![1.5f32.to_le_bytes().to_vec()],
        ..colors
    };

    assert_eq!(
        Ktx2Loader::load(&colors.bytes()).unwrap().format(),
        TextureFormat::Rgba8Srgb
    );
    assert_eq!(
        Ktx2Loader::load(&heights.bytes()).unwrap().format(),
        TextureFormat::R32Float
    );
}

#[test]
fn data_formats_cannot_be_read_as_colors() {
    let file = Ktx2 {
        format: VK_FORMAT_BC4_UNORM_BLOCK,
        width: 4,
        height: 4,
        faces: 1,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![vec![0; 8]],
    };

    assert!(
        Ktx2Loader::load(&file.bytes())
            .unwrap()
            .in_color_space(ColorSpace::Srgb)
            .is_err()
    );
}

#[test]
fn a_ktx2_level_of_the_wrong_size_is_rejected() {
    let file = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_UNORM,
        width: 2,
        height: 2,
        faces: 1,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![vec![0; 12]],
    };

    assert!(Ktx2Loader::load(&file.bytes()).is_err());
}

#[test]
fn a_dds_texture_reads_its_mip_chain() {
    let mut bytes = dds_header(8, 8, 2);
    bytes.extend(vec![1; 4 * BC7_BLOCK_BYTES]);
    bytes.extend(vec![2; BC7_BLOCK_BYTES]);
    let image = DdsLoader::load(&bytes)
        .unwrap()
        .in_color_space(ColorSpace::Srgb)
        .unwrap();

    assert_eq!(image.format(), TextureFormat::Bc7Srgb);
    assert_eq!(image.mip_levels(), 2);
    assert_eq!(image.layers().len(), 1);
    assert_eq!(image.layers()[0].len(), 5 * BC7_BLOCK_BYTES);
}

#[test]
fn a_png_reads_as_one_rgba_level() {
    let mut bytes = Vec::new();
    image::RgbaImage::from_raw(2, 1, vec![10, 20, 30, 255, 40, 50, 60, 255])
        .unwrap()
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();
    let image = PngLoader::load(&bytes)
        .unwrap()
        .in_color_space(ColorSpace::Linear)
        .unwrap();

    assert_eq!(image.format(), TextureFormat::Rgba8Unorm);
    assert_eq!(image.mip_levels(), 1);
    assert_eq!(image.layers(), &[vec![10, 20, 30, 255, 40, 50, 60, 255]]);
}

fn ktx2_error(file: Ktx2) -> String {
    Ktx2Loader::load(&file.bytes()).err().unwrap().to_string()
}

#[test]
fn a_ktx2_header_too_large_for_any_file_is_rejected_before_allocating() {
    let overflowing = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_UNORM,
        width: 1 << 31,
        height: 1 << 31,
        faces: 6,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![vec![0; 4]],
    };
    let unallocatable = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_UNORM,
        width: 1 << 30,
        height: 1 << 30,
        faces: 6,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![vec![0; 4]],
    };

    assert!(ktx2_error(overflowing).contains("too large"));
    assert!(ktx2_error(unallocatable).contains("cannot be allocated"));
}

#[test]
fn ktx2_mip_levels_beyond_the_full_chain_are_rejected() {
    let file = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_UNORM,
        width: 2,
        height: 2,
        faces: 1,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![vec![0; 16], vec![0; 4], vec![0; 4]],
    };

    assert!(ktx2_error(file).contains("at most 2"));
}

#[test]
fn a_ktx2_level_past_the_end_of_the_file_is_rejected() {
    let file = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_UNORM,
        width: 1,
        height: 1,
        faces: 1,
        supercompression: SUPERCOMPRESSION_NONE,
        levels: vec![vec![0; 4]],
    };
    let mut bytes = file.bytes();
    bytes[KTX2_HEADER_SIZE..KTX2_HEADER_SIZE + 8].copy_from_slice(&(u64::MAX - 2).to_le_bytes());

    assert!(
        Ktx2Loader::load(&bytes)
            .err()
            .unwrap()
            .to_string()
            .contains("truncated")
    );
}

#[test]
fn a_supercompressed_ktx2_level_declaring_another_size_is_rejected() {
    let file = Ktx2 {
        format: VK_FORMAT_R8G8B8A8_UNORM,
        width: 2,
        height: 2,
        faces: 1,
        supercompression: SUPERCOMPRESSION_ZSTANDARD,
        levels: vec![vec![0; 16]],
    };
    let mut bytes = file.bytes();
    let uncompressed_length = KTX2_HEADER_SIZE + 16;
    bytes[uncompressed_length..uncompressed_length + 8].copy_from_slice(&(1u64 << 60).to_le_bytes());

    assert!(
        Ktx2Loader::load(&bytes)
            .err()
            .unwrap()
            .to_string()
            .contains("expected 16")
    );
}

#[test]
fn a_dds_header_too_large_for_any_file_is_rejected() {
    assert!(DdsLoader::load(&dds_header(1 << 31, 1 << 31, 1)).is_err());
    assert!(DdsLoader::load(&dds_header(8, 8, 40)).is_err());
    assert!(DdsLoader::load(&dds_header(0, 8, 1)).is_err());
}

fn written(image: &TextureImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    Ktx2Writer::write(image, &mut bytes).unwrap();
    bytes
}

#[test]
fn a_written_cube_map_reads_back_with_every_level() {
    let layers: Vec<Vec<u8>> = (0..6u8)
        .map(|face| {
            let mut layer = tagged(0, face, 4 * 4 * 2);
            layer.extend(tagged(1, face, 2 * 2 * 2));
            layer.extend(tagged(2, face, 2));
            layer
        })
        .collect();
    let image = TextureImage::new(TextureFormat::Rg8Unorm, 4, 4, 3, layers.clone()).unwrap();
    let read = Ktx2Loader::load(&written(&image)).unwrap();

    assert_eq!(read.format(), TextureFormat::Rg8Unorm);
    assert_eq!((read.width(), read.height(), read.mip_levels()), (4, 4, 3));
    assert_eq!(read.layers(), layers.as_slice());
}

#[test]
fn every_uncompressed_format_reads_back_as_written() {
    for format in [
        TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8Srgb,
        TextureFormat::Rgba16Float,
        TextureFormat::R16Unorm,
        TextureFormat::Rg16Unorm,
        TextureFormat::R32Float,
    ] {
        let size = format.level_size(2, 1) + format.level_size(1, 1);
        let layer: Vec<u8> = (0..size as u8).collect();
        let image = TextureImage::new(format, 2, 1, 2, vec![layer.clone()]).unwrap();
        let read = Ktx2Loader::load(&written(&image)).unwrap();

        assert_eq!(read.format(), format, "{format}");
        assert_eq!(read.layers(), &[layer], "{format}");
    }
}

#[test]
fn a_written_file_describes_its_texels() {
    let image = TextureImage::new(TextureFormat::Rgba8Srgb, 1, 1, 1, vec![vec![0; 4]]).unwrap();
    let bytes = written(&image);
    let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    let descriptor = word(48) as usize;
    let block = descriptor + 4;
    let alpha_sample = block + 24 + 3 * 16;

    assert_eq!(word(16), 1, "typeSize");
    assert_eq!(word(descriptor) as usize, word(52) as usize);
    assert_eq!(&bytes[block + 8..block + 11], &[1, 1, 2], "RGBSDA, BT.709, sRGB");
    assert_eq!(bytes[block + 16], 4, "bytes per texel");
    assert_eq!(bytes[alpha_sample + 3], 0x1F, "linear alpha");
}

#[test]
fn block_compressed_textures_are_not_written() {
    let image = TextureImage::new(TextureFormat::Bc7Srgb, 4, 4, 1, vec![vec![0; BC7_BLOCK_BYTES]]).unwrap();

    assert!(Ktx2Writer::write(&image, &mut Vec::new()).is_err());
}

#[test]
fn a_texture_with_layers_of_the_wrong_size_is_refused() {
    assert!(TextureImage::new(TextureFormat::Rgba8Unorm, 2, 2, 1, vec![vec![0; 15]]).is_err());
    assert!(TextureImage::new(TextureFormat::Rgba8Unorm, 2, 2, 1, Vec::new()).is_err());
}
