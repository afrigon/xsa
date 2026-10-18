mod cube_face;
mod height_range;
mod level_record;
mod page_address;
mod page_encoding;
mod page_entry;
mod page_key;
mod terrain_header;
mod terrain_reader;

pub use cube_face::CubeFace;
pub use height_range::HeightRange;
pub use level_record::LevelRecord;
pub use page_address::PageAddress;
pub use page_encoding::PageEncoding;
pub use page_entry::PageEntry;
pub use page_key::PageKey;
pub use terrain_header::TerrainHeader;
pub use terrain_reader::TerrainReader;

/// The width and height of a page's own area, in texels: level 0's whole face.
pub const PAGE_INTERIOR: u32 = 128;
/// The texels copied from the neighbouring pages on every side of a page's own area.
pub const PAGE_BORDER: u32 = 4;
/// The width and height of a stored page, in texels: its own area and its border on both sides.
pub const PAGE_EXTENT: u32 = PAGE_INTERIOR + 2 * PAGE_BORDER;
