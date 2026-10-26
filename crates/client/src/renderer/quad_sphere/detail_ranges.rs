use std::f64::consts::SQRT_2;

use super::DEEPEST_DEPTH;

const TILE_GRID_CELLS: u32 = 32;
// Roots of distant bodies are drawn with 16, 8, 4 and down to 2 cells per side.
pub(in crate::renderer) const COARSEST_DETAIL: i32 = -4;
// A tile deeper than the roots is drawn at its depth or the level above: with each level's range at least four chord
// radii of the next level's tiles, a tile whose parent split always lies within the parent's range.
// The fraction of a level's range after which its vertices start morphing towards the coarser level's grid.
const MORPH_START_FRACTION: f64 = 0.75;
// Neighbors stay within one level of each other when a level's range exceeds four chord radii of the next level's
// tiles (at most √2 radii over 2^depth): 4 × √2 / 2 radii for level 0.
const MINIMUM_RANGE_RADII: f64 = 2.0 * SQRT_2;

// The distances at which each detail level is enough: tiles at least `fine_distance(detail)` away keep their
// vertices under the target spacing on screen. Every coarser level reaches twice as far.
pub(in crate::renderer) struct DetailRanges {
    root_distance: f64,
}

impl DetailRanges {
    pub fn new(radius: f64, focal_length_pixels: f64, vertex_spacing_pixels: f64) -> Self {
        let root_vertex_spacing = 2.0 * radius / f64::from(TILE_GRID_CELLS);
        let target_distance = root_vertex_spacing * focal_length_pixels / vertex_spacing_pixels;

        Self {
            root_distance: target_distance.max(MINIMUM_RANGE_RADII * radius),
        }
    }

    pub fn fine_distance(&self, detail: i32) -> f64 {
        self.root_distance * f64::from(-detail).exp2()
    }

    // The detail a tile needs when its nearest point is `nearest` away, or None when it must split: the coarsest its
    // distance allows, so a tile lying beyond its depth's range is drawn as that part of its parent is, and morphs as
    // its parent's neighbors do.
    pub fn detail(&self, depth: u32, nearest: f64) -> Option<i32> {
        (DetailRanges::coarsest_detail(depth)..=depth as i32).find(|&detail| nearest >= self.fine_distance(detail))
    }

    // The coarsest detail a tile at `depth` is drawn at: its parent's, or for a root the coarsest grid.
    pub fn coarsest_detail(depth: u32) -> i32 {
        if depth == 0 { COARSEST_DETAIL } else { depth as i32 - 1 }
    }

    // The finest detail drawn `distance` meters from the camera, no deeper than the deepest tiles.
    pub fn finest_detail(&self, distance: f64) -> i32 {
        let detail = (self.root_distance / distance).log2().floor();

        detail.clamp(f64::from(COARSEST_DETAIL), f64::from(DEEPEST_DEPTH)) as i32
    }

    // The level of a height map `extent` texels wide per face whose texels are as far apart as the vertices of
    // `detail`; finer than the map's level 0 below zero. Mirrored by `tileHeightLevels` in `terrain/tile.slang`.
    pub fn height_level(detail: i32, extent: u32) -> i32 {
        extent.ilog2() as i32 - TILE_GRID_CELLS.ilog2() as i32 - detail
    }

    // Each level coarser than its depth halves a tile's cells per side, so its vertices are as far apart as the
    // level's own tiles' are.
    pub fn grid_cells(depth: u32, detail: i32) -> u32 {
        TILE_GRID_CELLS >> (depth as i32 - detail)
    }

    // A tile is fully morphed where a coarser neighbor can start: at the coarser level's distance.
    pub fn morph_end(&self, detail: i32) -> f64 {
        if detail == COARSEST_DETAIL {
            return f64::INFINITY;
        }

        self.fine_distance(detail - 1)
    }

    pub fn morph_start(&self, detail: i32) -> f64 {
        self.morph_end(detail) * MORPH_START_FRACTION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADIUS: f64 = 1000.0;
    const FOCAL_LENGTH_PIXELS: f64 = 1000.0;
    const VERTEX_SPACING_PIXELS: f64 = 8.0;

    fn ranges() -> DetailRanges {
        DetailRanges::new(RADIUS, FOCAL_LENGTH_PIXELS, VERTEX_SPACING_PIXELS)
    }

    #[test]
    fn each_level_reaches_half_as_far_as_the_coarser_one() {
        let ranges = ranges();

        for detail in COARSEST_DETAIL + 1..10 {
            assert_eq!(ranges.fine_distance(detail - 1), 2.0 * ranges.fine_distance(detail));
        }
    }

    #[test]
    fn vertices_at_the_fine_distance_are_the_target_spacing_apart() {
        let ranges = ranges();
        let vertex_spacing = 2.0 * RADIUS / f64::from(TILE_GRID_CELLS) / 8.0;
        let pixels = vertex_spacing / ranges.fine_distance(3) * FOCAL_LENGTH_PIXELS;

        assert!((pixels - VERTEX_SPACING_PIXELS).abs() < 1e-9);
    }

    #[test]
    fn coarse_spacing_is_limited_to_keep_neighbors_within_one_level() {
        let ranges = DetailRanges::new(RADIUS, FOCAL_LENGTH_PIXELS, 1000.0);

        assert_eq!(ranges.fine_distance(0), MINIMUM_RANGE_RADII * RADIUS);
    }

    #[test]
    fn distant_roots_take_the_coarsest_grid_that_is_enough() {
        let ranges = ranges();

        assert_eq!(ranges.detail(0, ranges.fine_distance(0)), Some(0));
        assert_eq!(ranges.detail(0, ranges.fine_distance(-2) * 1.5), Some(-2));
        assert_eq!(ranges.detail(0, f64::MAX), Some(COARSEST_DETAIL));
        assert_eq!(ranges.detail(0, ranges.fine_distance(0) / 2.0), None);
    }

    #[test]
    fn deeper_tiles_split_inside_their_range() {
        let ranges = ranges();

        assert_eq!(ranges.detail(4, ranges.fine_distance(4)), Some(4));
        assert_eq!(ranges.detail(4, ranges.fine_distance(4) * 0.99), None);
    }

    #[test]
    fn tiles_beyond_their_range_take_their_parents_level() {
        let ranges = ranges();

        assert_eq!(ranges.detail(4, ranges.fine_distance(3)), Some(3));
        assert_eq!(ranges.detail(4, ranges.fine_distance(3) * 0.99), Some(4));
    }

    #[test]
    fn grids_halve_with_each_coarser_level() {
        assert_eq!(DetailRanges::grid_cells(3, 3), TILE_GRID_CELLS);
        assert_eq!(DetailRanges::grid_cells(3, 2), TILE_GRID_CELLS / 2);
        assert_eq!(DetailRanges::grid_cells(0, 0), TILE_GRID_CELLS);
        assert_eq!(DetailRanges::grid_cells(0, -1), TILE_GRID_CELLS / 2);
        assert_eq!(DetailRanges::grid_cells(0, COARSEST_DETAIL), 2);
    }

    #[test]
    fn a_detail_reads_the_height_level_its_vertex_spacing_matches() {
        assert_eq!(DetailRanges::height_level(0, 4096), 7);
        assert_eq!(DetailRanges::height_level(7, 4096), 0);
        assert_eq!(DetailRanges::height_level(COARSEST_DETAIL, 4096), 11);
        assert_eq!(DetailRanges::height_level(9, 4096), -2);
    }

    #[test]
    fn the_finest_detail_at_a_distance_is_the_one_whose_range_starts_there() {
        let ranges = ranges();

        assert_eq!(ranges.finest_detail(ranges.fine_distance(5)), 5);
        assert_eq!(ranges.finest_detail(ranges.fine_distance(5) * 1.5), 4);
        assert_eq!(ranges.finest_detail(0.0), DEEPEST_DEPTH as i32);
    }

    #[test]
    fn the_coarsest_level_never_morphs() {
        assert_eq!(ranges().morph_start(COARSEST_DETAIL), f64::INFINITY);
    }
}
