use ash::vk;

use super::geometry::Geometry;

#[derive(Clone, Copy)]
pub(in crate::renderer) enum TileSurface {
    Smooth,
    Displaced { terrain_data: vk::DeviceAddress },
}

impl TileSurface {
    pub fn geometry(self) -> Geometry {
        match self {
            TileSurface::Smooth => Geometry::SmoothTerrain,
            TileSurface::Displaced { .. } => Geometry::Terrain,
        }
    }

    pub fn terrain_data(self) -> Option<vk::DeviceAddress> {
        match self {
            TileSurface::Smooth => None,
            TileSurface::Displaced { terrain_data } => Some(terrain_data),
        }
    }
}
