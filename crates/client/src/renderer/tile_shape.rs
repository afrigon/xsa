// How a terrain's tiles are placed: on its sphere until its heights' file is loaded, then displaced by its heights.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) enum TileShape {
    Smooth,
    Displaced,
}

impl TileShape {
    pub const ALL: [TileShape; 2] = [TileShape::Smooth, TileShape::Displaced];

    // Matches `TileShape` in terrain/vertex.slang.
    pub fn shader_id(self) -> u32 {
        match self {
            TileShape::Smooth => 0,
            TileShape::Displaced => 1,
        }
    }
}
