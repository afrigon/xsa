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

impl TerrainRegion {
    /// The parts naming the region in a file name, coarsest first: its face, level and page, when selected.
    pub fn file_name_parts(&self) -> Vec<String> {
        let level = |levels: &LevelSelection| match levels {
            LevelSelection::Every => None,
            LevelSelection::One(level) => Some(format!("level-{level}")),
        };

        match self {
            TerrainRegion::Cube(levels) => level(levels).into_iter().collect(),
            TerrainRegion::Face(face, levels) => {
                [Some(face.to_string()), level(levels)].into_iter().flatten().collect()
            }
            TerrainRegion::Page(address) => vec![
                address.face.to_string(),
                format!("level-{}", address.level),
                format!("page-{}-{}", address.key.x(), address.key.y()),
            ],
        }
    }
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

#[cfg(test)]
mod tests {
    use resource_loader::terrain::PageKey;

    use super::*;

    #[test]
    fn a_region_names_what_it_selects() {
        let parts = |region: TerrainRegion| region.file_name_parts().join(".");

        assert_eq!(parts(TerrainRegion::Cube(LevelSelection::Every)), "");
        assert_eq!(parts(TerrainRegion::Cube(LevelSelection::One(3))), "level-3");
        assert_eq!(
            parts(TerrainRegion::Face(CubeFace::PositiveX, LevelSelection::Every)),
            "+x"
        );
        assert_eq!(
            parts(TerrainRegion::Page(PageAddress {
                face: CubeFace::NegativeX,
                level: 3,
                key: PageKey::from_coordinates(1, 2),
            })),
            "-x.level-3.page-1-2"
        );
    }
}
