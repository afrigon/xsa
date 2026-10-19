#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) enum Geometry {
    SmoothTerrain,
    Terrain,
}

impl Geometry {
    pub const ALL: &[Geometry] = &[Geometry::SmoothTerrain, Geometry::Terrain];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn path(self) -> &'static str {
        match self {
            Geometry::SmoothTerrain => "smooth-terrain",
            Geometry::Terrain => "terrain",
        }
    }
}
