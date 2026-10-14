use std::f64::consts::SQRT_2;

const TILE_GRID_CELLS: u32 = 32;
// Roots of distant bodies are drawn with 16, 8, 4 and down to 2 cells per side.
pub(in crate::renderer) const COARSEST_DETAIL: i32 = -4;
// The fraction of a level's range after which its vertices start morphing towards the coarser level's grid.
const MORPH_START_FRACTION: f64 = 0.75;
// Neighbors stay within one level of each other when a level's range exceeds four chord radii of the next level's
// tiles (at most √2 radii over 2^depth): 4 × √2 / 2 radii for level 0.
const MINIMUM_RANGE_RADII: f64 = 2.0 * SQRT_2;

// The distances at which each detail level is enough: tiles at least `fine_distance(detail)` away keep their
// vertices under the target spacing on screen. Every coarser level reaches twice as far.
pub(in crate::renderer) struct DetailRanges {
    root_distance: f64,
    vertex_spacing_angle: f64,
}

impl DetailRanges {
    pub fn new(radius: f64, focal_length_pixels: f64, vertex_spacing_pixels: f64) -> Self {
        let root_vertex_spacing = 2.0 * radius / f64::from(TILE_GRID_CELLS);
        let target_distance = root_vertex_spacing * focal_length_pixels / vertex_spacing_pixels;

        Self {
            root_distance: target_distance.max(MINIMUM_RANGE_RADII * radius),
            vertex_spacing_angle: vertex_spacing_pixels / focal_length_pixels,
        }
    }

    // The angle the target vertex spacing covers on screen; the shader picks height mip levels from the same angle.
    pub fn vertex_spacing_angle(&self) -> f64 {
        self.vertex_spacing_angle
    }

    pub fn fine_distance(&self, detail: i32) -> f64 {
        self.root_distance * f64::from(-detail).exp2()
    }

    // The detail a tile needs when its nearest point is `nearest` away, or None when it must split.
    pub fn detail(&self, depth: u32, nearest: f64) -> Option<i32> {
        if depth > 0 {
            let detail = depth as i32;
            return (nearest >= self.fine_distance(detail)).then_some(detail);
        }

        (COARSEST_DETAIL..=0).find(|&detail| nearest >= self.fine_distance(detail))
    }

    pub fn grid_cells(detail: i32) -> u32 {
        TILE_GRID_CELLS >> detail.min(0).unsigned_abs()
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
    fn grids_halve_below_level_zero() {
        assert_eq!(DetailRanges::grid_cells(3), TILE_GRID_CELLS);
        assert_eq!(DetailRanges::grid_cells(0), TILE_GRID_CELLS);
        assert_eq!(DetailRanges::grid_cells(-1), TILE_GRID_CELLS / 2);
        assert_eq!(DetailRanges::grid_cells(COARSEST_DETAIL), 2);
    }

    #[test]
    fn the_coarsest_level_never_morphs() {
        assert_eq!(ranges().morph_start(COARSEST_DETAIL), f64::INFINITY);
    }
}
