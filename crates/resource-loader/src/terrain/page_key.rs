use std::str::FromStr;

use anyhow::Context;

/// A page's place within its face and level: the Morton code of its column and row, the column in the even bits.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PageKey {
    pub code: u64,
}

impl PageKey {
    const EVEN_BITS: u64 = 0x5555_5555_5555_5555;

    pub fn from_coordinates(x: u32, y: u32) -> PageKey {
        PageKey {
            code: Self::spread(x) | Self::spread(y) << 1,
        }
    }

    /// The page's column, left to right from 0.
    pub fn x(self) -> u32 {
        Self::gather(self.code)
    }

    /// The page's row, top to bottom from 0.
    pub fn y(self) -> u32 {
        Self::gather(self.code >> 1)
    }

    /// The page one level coarser that covers this one.
    pub fn parent(self) -> PageKey {
        PageKey { code: self.code >> 2 }
    }

    fn spread(coordinate: u32) -> u64 {
        let mut bits = u64::from(coordinate);
        bits = (bits | bits << 16) & 0x0000_FFFF_0000_FFFF;
        bits = (bits | bits << 8) & 0x00FF_00FF_00FF_00FF;
        bits = (bits | bits << 4) & 0x0F0F_0F0F_0F0F_0F0F;
        bits = (bits | bits << 2) & 0x3333_3333_3333_3333;
        (bits | bits << 1) & Self::EVEN_BITS
    }

    fn gather(bits: u64) -> u32 {
        let mut bits = bits & Self::EVEN_BITS;
        bits = (bits | bits >> 1) & 0x3333_3333_3333_3333;
        bits = (bits | bits >> 2) & 0x0F0F_0F0F_0F0F_0F0F;
        bits = (bits | bits >> 4) & 0x00FF_00FF_00FF_00FF;
        bits = (bits | bits >> 8) & 0x0000_FFFF_0000_FFFF;
        ((bits | bits >> 16) & 0xFFFF_FFFF) as u32
    }
}

/// Reads a page key from its column and row: `x,y`.
impl FromStr for PageKey {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> anyhow::Result<PageKey> {
        let (x, y) = text
            .split_once(',')
            .with_context(|| format!("page {text:?}: expected <column>,<row>"))?;
        let coordinate = |value: &str| {
            value
                .trim()
                .parse::<u32>()
                .with_context(|| format!("page {text:?}: {value:?} is not a page coordinate"))
        };

        Ok(PageKey::from_coordinates(coordinate(x)?, coordinate(y)?))
    }
}
