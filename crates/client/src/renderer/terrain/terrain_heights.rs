use super::TexelFootprint;
use crate::renderer::quad_sphere::{HeightRange, QuadSphereTile, TileHeightBounds};
use crate::resource_kinds::{HeightMap, TexelRange};

// A terrain's resident heights as tile selection bounds them, in meters relative to the body's radius.
pub(in crate::renderer) struct TerrainHeights<'a> {
    pub map: &'a HeightMap,
    pub footprint: TexelFootprint,
}

impl TerrainHeights<'_> {
    fn meters(&self, range: TexelRange) -> HeightRange {
        let level = self.map.level();
        let meters =
            |value: u16| level.minimum() + f64::from(value) / f64::from(u16::MAX) * (level.maximum() - level.minimum());

        HeightRange {
            minimum: meters(range.minimum),
            maximum: meters(range.maximum),
        }
    }
}

impl TileHeightBounds for TerrainHeights<'_> {
    fn overall_range(&self) -> Option<HeightRange> {
        Some(self.meters(self.map.bounds().overall()))
    }

    // Vertices no farther than `farthest` sample levels no coarser than the one whose texels are as far apart as the
    // vertices there, and none finer than the finest resident.
    fn tile_range(&self, tile: QuadSphereTile, farthest: f64) -> Option<HeightRange> {
        let level = self.map.level().level();
        let coarsest = self
            .footprint
            .height_level(self.map.extent(), farthest)
            .ceil()
            .max(f64::from(level)) as u32;

        Some(self.meters(self.map.bounds().range(tile.face, tile.depth, tile.x, tile.y, coarsest)))
    }
}
