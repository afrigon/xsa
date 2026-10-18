use std::io::{self, Write};

use super::{HeightRange, PageEncoding, PageKey};

/// A present page: where its data is and, for page heights, the range its values span.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PageEntry {
    pub key: PageKey,
    pub data_offset: u64,
    pub data_size: u32,
    pub range: Option<HeightRange>,
}

impl PageEntry {
    const DATA_OFFSET_OFFSET: usize = 8;
    const DATA_SIZE_OFFSET: usize = 16;
    const MINIMUM_OFFSET: usize = 20;
    const MAXIMUM_OFFSET: usize = 24;
    const DIRECT_SIZE: usize = 20;
    const PAGE_HEIGHTS_SIZE: usize = 28;

    /// The size of an entry in bytes, in a file of that encoding.
    pub fn size(encoding: PageEncoding) -> usize {
        match encoding {
            PageEncoding::Direct => Self::DIRECT_SIZE,
            PageEncoding::PageHeights => Self::PAGE_HEIGHTS_SIZE,
        }
    }

    /// Reads an entry of `PageEntry::size(encoding)` bytes.
    pub fn read(bytes: &[u8], encoding: PageEncoding) -> PageEntry {
        let long = |offset: usize| u64::from_le_bytes(bytes[offset..offset + size_of::<u64>()].try_into().unwrap());
        let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + size_of::<u32>()].try_into().unwrap());

        PageEntry {
            key: PageKey(long(0)),
            data_offset: long(Self::DATA_OFFSET_OFFSET),
            data_size: word(Self::DATA_SIZE_OFFSET),
            range: match encoding {
                PageEncoding::Direct => None,
                PageEncoding::PageHeights => Some(HeightRange {
                    minimum: f32::from_bits(word(Self::MINIMUM_OFFSET)),
                    maximum: f32::from_bits(word(Self::MAXIMUM_OFFSET)),
                }),
            },
        }
    }

    pub fn write(&self, output: &mut impl Write) -> io::Result<()> {
        output.write_all(&self.key.0.to_le_bytes())?;
        output.write_all(&self.data_offset.to_le_bytes())?;
        output.write_all(&self.data_size.to_le_bytes())?;

        if let Some(range) = self.range {
            output.write_all(&range.minimum.to_le_bytes())?;
            output.write_all(&range.maximum.to_le_bytes())?;
        }

        Ok(())
    }
}
