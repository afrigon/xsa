use texture_image::{ColorSpace, DdsLoader, Ktx2Loader, PngLoader, TextureFormat, TextureLoader};

const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
const KTX2_HEADER_SIZE: usize = 80;
const KTX2_LEVEL_ENTRY_SIZE: usize = 24;
const VK_FORMAT_R8G8B8A8_UNORM: u32 = 37;
const VK_FORMAT_R16_UNORM: u32 = 70;
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
    let image = Ktx2Loader::load(&file.bytes(), ColorSpace::Srgb).unwrap();

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
    let image = Ktx2Loader::load(&file.bytes(), ColorSpace::Linear).unwrap();

    assert_eq!(image.format(), TextureFormat::R16Unorm);
    assert_eq!((image.width(), image.height()), (4, 2));
    assert_eq!(image.layers(), &[(0..16).collect::<Vec<u8>>()]);
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

    assert!(Ktx2Loader::load(&file.bytes(), ColorSpace::Srgb).is_err());
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

    assert!(Ktx2Loader::load(&file.bytes(), ColorSpace::Linear).is_err());
}

#[test]
fn a_dds_texture_reads_its_mip_chain() {
    let mut bytes = vec![0; DDS_HEADER_SIZE];
    bytes[..4].copy_from_slice(b"DDS ");
    bytes[12..16].copy_from_slice(&8u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
    bytes[28..32].copy_from_slice(&2u32.to_le_bytes());
    bytes[84..88].copy_from_slice(b"DX10");
    bytes[128..132].copy_from_slice(&DXGI_FORMAT_BC7_UNORM.to_le_bytes());
    bytes.extend(vec![1; 4 * BC7_BLOCK_BYTES]);
    bytes.extend(vec![2; BC7_BLOCK_BYTES]);
    let image = DdsLoader::load(&bytes, ColorSpace::Srgb).unwrap();

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
    let image = PngLoader::load(&bytes, ColorSpace::Linear).unwrap();

    assert_eq!(image.format(), TextureFormat::Rgba8Unorm);
    assert_eq!(image.mip_levels(), 1);
    assert_eq!(image.layers(), &[vec![10, 20, 30, 255, 40, 50, 60, 255]]);
}
