// A layer of a terrain that shading reads, each streamed from its own `.terrain` file; its index is its place in a
// tile's pages.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TerrainLayer {
    Color,
    Normal,
    Medium,
    Opposition,
}

impl TerrainLayer {
    pub const ALL: [TerrainLayer; 4] = [
        TerrainLayer::Color,
        TerrainLayer::Normal,
        TerrainLayer::Medium,
        TerrainLayer::Opposition,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            TerrainLayer::Color => "color",
            TerrainLayer::Normal => "normal",
            TerrainLayer::Medium => "hapke medium",
            TerrainLayer::Opposition => "hapke opposition",
        }
    }
}
