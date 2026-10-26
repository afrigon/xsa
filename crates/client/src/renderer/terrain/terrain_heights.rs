use resource_vulkan::StreamedMips;

use crate::renderer::quad_sphere::{DetailRanges, HeightRange, QuadSphereTile, TileHeightBounds};
use crate::resource_kinds::{HeightMap, TexelRange};

// A terrain's resident heights as tile selection bounds them, in meters relative to the body's radius.
pub(in crate::renderer) struct TerrainHeights<'a> {
    pub map: &'a HeightMap,
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

    // A tile drawn at its coarsest detail reads that detail's level and, as it morphs, the next coarser: the coarsest
    // level it can read, within the resident ones.
    fn tile_range(&self, tile: QuadSphereTile) -> Option<HeightRange> {
        let coarsest_detail = DetailRanges::coarsest_detail(tile.depth);
        let level = DetailRanges::height_level(coarsest_detail, self.map.extent()) + 1;
        let resident = self.map.first_level() as i32..=self.map.mip_levels() as i32 - 1;
        let coarsest = level.clamp(*resident.start(), *resident.end()) as u32;

        Some(self.meters(self.map.bounds().range(tile.face, tile.depth, tile.x, tile.y, coarsest)))
    }
}
