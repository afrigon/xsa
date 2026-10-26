// A map of a terrain that shading reads, besides its heights; its index is its slot in the terrain's data.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TerrainLayer {
    Color,
    Normal,
    Scattering,
    Surge,
}

impl TerrainLayer {
    pub const ALL: [TerrainLayer; 4] = [
        TerrainLayer::Color,
        TerrainLayer::Normal,
        TerrainLayer::Scattering,
        TerrainLayer::Surge,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            TerrainLayer::Color => "color",
            TerrainLayer::Normal => "normal",
            TerrainLayer::Scattering => "scattering",
            TerrainLayer::Surge => "surge",
        }
    }
}
