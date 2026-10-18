use std::fmt;

use resource_loader::terrain::PageAddress;
use resource_loader::utils::CubeFace;

use super::LevelSelection;

/// The part of a `.terrain` file an extract writes out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainRegion {
    /// All six faces, as a cube map.
    Cube(LevelSelection),
    /// One face, as a 2D texture.
    Face(CubeFace, LevelSelection),
    /// One page, its border included.
    Page(PageAddress),
}

impl fmt::Display for TerrainRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TerrainRegion::Cube(levels) => write!(formatter, "the cube map at {levels}"),
            TerrainRegion::Face(face, levels) => write!(formatter, "face {face} at {levels}"),
            TerrainRegion::Page(address) => write!(
                formatter,
                "{} level {} page {},{}",
                address.face,
                address.level,
                address.key.x(),
                address.key.y()
            ),
        }
    }
}
