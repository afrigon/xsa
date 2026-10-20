// How a surface's colour combines with the terrain's colour layer up close.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SurfaceTint {
    // The surface divided by its average colour, multiplying the layer: the body's colour, with the surface's own
    // variation.
    Relative,
    // The surface at an average brightness of 1, multiplying the layer: its photograph's colour cast is kept.
    AsIs,
    // The surface's colour instead of the layer's.
    Replace,
}

impl SurfaceTint {
    // Matches `SurfaceTint` in terrain/surface/tint.slang.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            SurfaceTint::Relative => 0,
            SurfaceTint::AsIs => 1,
            SurfaceTint::Replace => 2,
        }
    }
}
