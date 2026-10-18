use std::io::Cursor;

use resource_loader::terrain::{
    CubeFace, HeightRange, LevelRecord, PAGE_EXTENT, PageEncoding, PageEntry, PageKey, TerrainHeader, TerrainReader,
};
use resource_loader::texture::TextureFormat;

#[test]
fn a_page_key_interleaves_its_column_in_the_even_bits() {
    assert_eq!(PageKey::from_coordinates(1, 0).0, 0b01);
    assert_eq!(PageKey::from_coordinates(0, 1).0, 0b10);
    assert_eq!(PageKey::from_coordinates(3, 2).0, 0b1101);

    let key = PageKey::from_coordinates(12345, 678);
    assert_eq!((key.x(), key.y()), (12345, 678));
    assert_eq!(key.parent(), PageKey::from_coordinates(12345 / 2, 678 / 2));
}

#[test]
fn page_heights_quantise_within_half_a_step() {
    let range = HeightRange::spanning([-9000.0, 3.25, 11000.0]).unwrap();
    let step = (range.maximum - range.minimum) / f32::from(u16::MAX);

    for height in [-9000.0, -1234.5, 0.0, 10999.0, 11000.0] {
        assert!((range.height(range.value(height)) - height).abs() <= step / 2.0 + 1e-3);
    }
}

#[test]
fn a_flat_page_stores_its_one_height_as_both_bounds() {
    let range = HeightRange::spanning([42.0; 4]).unwrap();

    assert_eq!((range.minimum, range.maximum), (42.0, 42.0));
    assert_eq!(range.height(range.value(42.0)), 42.0);
}

#[test]
fn every_face_looks_outward_with_columns_and_rows_in_vulkan_order() {
    for face in CubeFace::ALL {
        assert_eq!(face.column_axis().cross(face.row_axis()), -face.normal(), "{face:?}");
        assert_eq!(CubeFace::from_normal(face.normal()), Some(face));
    }
}

#[test]
fn a_header_reads_back_what_it_wrote() {
    let header = TerrainHeader {
        format: TextureFormat::R16Unorm,
        encoding: PageEncoding::PageHeights,
        level_count: 5,
    };
    let mut bytes = Vec::new();
    header.write(&mut bytes).unwrap();

    assert_eq!(&bytes[..4], b"XSAT");
    assert_eq!(
        TerrainHeader::read(bytes.as_slice().try_into().unwrap()).unwrap(),
        header
    );

    bytes[4] = 2;
    assert!(TerrainHeader::read(bytes.as_slice().try_into().unwrap()).is_err());
}

#[test]
fn the_reader_finds_pages_by_key_and_decompresses_them() {
    let header = TerrainHeader {
        format: TextureFormat::R16Unorm,
        encoding: PageEncoding::PageHeights,
        level_count: 2,
    };
    let entry_size = PageEntry::size(header.encoding);
    let record_count = CubeFace::ALL.len() * header.level_count as usize;
    let entries_start = (TerrainHeader::SIZE + record_count * LevelRecord::SIZE) as u64;
    let keys = [PageKey::from_coordinates(0, 0), PageKey::from_coordinates(1, 1)];
    let page = vec![7; header.page_size()];
    let compressed = zstd::bulk::compress(&page, 3).unwrap();
    let data_start = entries_start + (keys.len() * entry_size) as u64;
    let mut bytes = Vec::new();
    header.write(&mut bytes).unwrap();

    for face in CubeFace::ALL {
        for level in 0..header.level_count {
            let record = if face == CubeFace::NegativeY && level == 1 {
                LevelRecord {
                    entries_offset: entries_start,
                    entry_count: keys.len() as u64,
                }
            } else {
                LevelRecord {
                    entries_offset: entries_start,
                    entry_count: 0,
                }
            };
            record.write(&mut bytes).unwrap();
        }
    }

    for (index, key) in keys.into_iter().enumerate() {
        PageEntry {
            key,
            data_offset: data_start + (index * compressed.len()) as u64,
            data_size: compressed.len() as u32,
            range: Some(HeightRange {
                minimum: index as f32,
                maximum: 10.0,
            }),
        }
        .write(&mut bytes)
        .unwrap();
    }

    bytes.extend(&compressed);
    bytes.extend(&compressed);
    let mut reader = TerrainReader::new(Cursor::new(bytes)).unwrap();

    assert_eq!(reader.entries(CubeFace::NegativeY, 1).unwrap().len(), 2);
    assert!(reader.entries(CubeFace::PositiveX, 0).unwrap().is_empty());
    assert!(
        reader
            .entry(CubeFace::NegativeY, 1, PageKey::from_coordinates(1, 0))
            .unwrap()
            .is_none()
    );

    let entry = reader.entry(CubeFace::NegativeY, 1, keys[1]).unwrap().unwrap();
    assert_eq!(entry.range.unwrap().minimum, 1.0);
    assert_eq!(reader.page(&entry).unwrap(), page);
    assert_eq!(header.page_size(), (PAGE_EXTENT * PAGE_EXTENT) as usize * 2);
}
