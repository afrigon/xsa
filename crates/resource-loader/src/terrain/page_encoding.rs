use anyhow::bail;

/// How a page's texel values map to the quantities they hold.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageEncoding {
    /// Values as stored.
    Direct,
    /// 16-bit heights spread over each page's own `HeightRange`, kept in its entry.
    PageHeights,
}

impl PageEncoding {
    const DIRECT: u32 = 0;
    const PAGE_HEIGHTS: u32 = 1;

    pub fn from_value(value: u32) -> anyhow::Result<PageEncoding> {
        Ok(match value {
            Self::DIRECT => PageEncoding::Direct,
            Self::PAGE_HEIGHTS => PageEncoding::PageHeights,
            other => bail!("unknown page encoding {other}"),
        })
    }

    pub fn value(self) -> u32 {
        match self {
            PageEncoding::Direct => Self::DIRECT,
            PageEncoding::PageHeights => Self::PAGE_HEIGHTS,
        }
    }
}
