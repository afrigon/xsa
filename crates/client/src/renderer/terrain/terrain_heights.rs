use glam::{DVec2, IVec2};
use resource_loader::terrain::{PAGE_INTERIOR, PageAddress, PageKey};
use resource_loader::utils::CubeFace;
use resource_vulkan::StreamedSet;
use xsa_core::terrain::{HeightPage, HeightPages};

use super::{HeightPageSource, PageRanges};
use crate::renderer::gpu_data::HEIGHT_LEVELS;
use crate::renderer::quad_sphere::{HeightRange, QuadSphereTile, TileHeightBounds};

// The levels a tile's vertices read heights at sit this far coarser than the tile, finest last.
const HEIGHT_LEVEL_LEAD: u32 = 1;
// The cubic B-spline reads this many texels on each side of a sample's cell.
const SPLINE_REACH: i32 = 2;

// A terrain's heights as far as they are loaded: the pages its height set keeps in RAM, and the range of every page
// read. Meters relative to the body's radius.
pub(in crate::renderer) struct TerrainHeights<'a> {
    pub set: &'a StreamedSet<HeightPageSource>,
    pub ranges: &'a PageRanges,
    pub finest_level: u32,
}

impl HeightPages for TerrainHeights<'_> {
    fn finest_level(&self) -> u32 {
        self.finest_level
    }

    fn page(&self, address: PageAddress) -> Option<(PageAddress, HeightPage)> {
        let (found, values) = self.set.bytes(address)?;
        let range = self.ranges.get(found)?;

        Some((found, HeightPage { range, values }))
    }
}

impl TerrainHeights<'_> {
    // The levels the vertices of a tile at `depth` read, coarsest first; the shader picks between them by distance.
    pub fn levels(&self, depth: u32) -> [u32; HEIGHT_LEVELS] {
        let finest = i64::from(depth) - i64::from(HEIGHT_LEVEL_LEAD);

        std::array::from_fn(|index| {
            (finest - (HEIGHT_LEVELS - 1 - index) as i64).clamp(0, i64::from(self.finest_level)) as u32
        })
    }

    fn finest_range(&self, tile: QuadSphereTile) -> Option<HeightRange> {
        let address = tile.page(self.finest_level);
        let resident = self.set.page(address)?;
        let (found, page) = self.page(address)?;

        if resident.page != address || found != address {
            return None;
        }

        let pages_per_tile = f64::from(tile.depth - self.finest_level).exp2().recip();
        let interior = f64::from(PAGE_INTERIOR);
        let start = (DVec2::new(f64::from(tile.x), f64::from(tile.y)) * pages_per_tile
            - DVec2::new(f64::from(address.key.x()), f64::from(address.key.y())))
            * interior;
        let end = start + pages_per_tile * interior;
        let first = (start - 0.5).floor().as_ivec2() - IVec2::splat(SPLINE_REACH - 1);
        let last = (end - 0.5).floor().as_ivec2() + IVec2::splat(SPLINE_REACH);

        Some(HeightRange::from(page.range_over(first, last)))
    }
}

impl TileHeightBounds for TerrainHeights<'_> {
    // The combined range of the six level 0 pages, known once the file's header is loaded.
    fn overall_range(&self) -> Option<HeightRange> {
        CubeFace::ALL
            .into_iter()
            .try_fold(None, |range: Option<HeightRange>, face| {
                let page = self.ranges.get(PageAddress {
                    face,
                    level: 0,
                    key: PageKey::from_coordinates(0, 0),
                })?;

                let page = HeightRange::from(page);
                Some(Some(range.map_or(page, |range| range.union(page))))
            })?
    }

    // Bounds every height the shader can give the tile's vertices: the heights of the pages it reads, or of the
    // coarser pages standing in for them. A tile reading the finest level only, resident and in RAM, is bounded by
    // the texels its samples reach.
    fn tile_range(&self, tile: QuadSphereTile) -> Option<HeightRange> {
        let levels = self.levels(tile.depth);

        if levels[0] == self.finest_level
            && let Some(range) = self.finest_range(tile)
        {
            return Some(range);
        }

        levels.into_iter().try_fold(None, |range: Option<HeightRange>, level| {
            let (_, page) = self.ranges.nearest(tile.page(level))?;
            let page = HeightRange::from(page);

            Some(Some(range.map_or(page, |range| range.union(page))))
        })?
    }
}
