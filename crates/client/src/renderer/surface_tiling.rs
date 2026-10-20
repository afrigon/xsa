// How a surface lays its texture across the ground at each repeat size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SurfaceTiling {
    // Hexagonal cells, each turned and shifted, blended by height where they meet.
    Hex,
    // One plain repeat per size, shifted by a seeded offset.
    Offset,
}

impl SurfaceTiling {
    // Matches `SurfaceTiling` in terrain/surface/tiling.slang.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            SurfaceTiling::Hex => 0,
            SurfaceTiling::Offset => 1,
        }
    }
}
