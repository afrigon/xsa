use glam::{DMat2, DVec2, IVec2, Vec2};

// The hexagonal cells' centres are the corners of a grid of equilateral triangles: skewing the plane by this matrix
// puts them on the integer lattice. Mirrored by `skewedFromGrid` in terrain/surface/hex-tiling.slang.
const SKEWED_FROM_GRID: DMat2 = DMat2::from_cols_array(&[1.0, 0.0, -0.577_350_269_189_625_8, 1.154_700_538_379_251_5]);

// Where the camera stands on a lattice of hexagonal cells, skewed: its lattice cell and the fraction across
// it, so pixels add only their small offset in f32. Mirrored by `HexAnchor` in terrain/surface/surface-data.slang.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(in crate::renderer) struct HexAnchor {
    pub cell: IVec2,
    pub fraction: Vec2,
}

impl HexAnchor {
    pub const NONE: HexAnchor = HexAnchor {
        cell: IVec2::ZERO,
        fraction: Vec2::ZERO,
    };

    // `place` is in cells.
    pub fn at(place: DVec2) -> HexAnchor {
        let skewed = SKEWED_FROM_GRID * place;
        let cell = skewed.floor();

        HexAnchor {
            cell: cell.as_ivec2(),
            fraction: (skewed - cell).as_vec2(),
        }
    }
}
