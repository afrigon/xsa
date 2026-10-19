use super::geometry::Geometry;

// How a terrain's tiles are placed: on its sphere until its heights' file is loaded, then displaced by its heights.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::renderer) enum TileShape {
    Smooth,
    Displaced,
}

impl TileShape {
    pub fn geometry(self) -> Geometry {
        match self {
            TileShape::Smooth => Geometry::SmoothTerrain,
            TileShape::Displaced => Geometry::Terrain,
        }
    }
}
