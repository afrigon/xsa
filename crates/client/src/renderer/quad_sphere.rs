mod detail_ranges;
mod height_range;
mod horizon_culler;
mod selected_tile;
mod tile_height_bounds;
mod tile_selection;

pub(in crate::renderer) use detail_ranges::{COARSEST_DETAIL, DetailRanges};
pub(in crate::renderer) use height_range::HeightRange;
pub(in crate::renderer) use horizon_culler::HorizonCuller;
pub(in crate::renderer) use selected_tile::SelectedTile;
pub(in crate::renderer) use tile_height_bounds::TileHeightBounds;
use tile_selection::TileSelection;

use glam::{DQuat, DVec3};
use xsa_core::terrain::QuadSphereTile;

use super::culler::Culler;

// A tile's grid points sit at offsets from its center that f32 holds exactly down to this depth.
pub(in crate::renderer) const DEEPEST_DEPTH: u32 = 22;

// A sphere whose cube faces split into quadtrees of tiles, finer where the camera is closer.
pub(in crate::renderer) struct QuadSphere<'a> {
    pub radius: f64,
    // None for a smooth sphere.
    pub height: Option<&'a dyn TileHeightBounds>,
    // How far surface displacement can move the ground either way beyond its heights (m).
    pub relief: f64,
    // Multiplies the heights as they are drawn.
    pub height_scale: f64,
    // Relative to the camera.
    pub center: DVec3,
    pub orientation: DQuat,
}

impl QuadSphere<'_> {
    pub fn select(&self, ranges: &DetailRanges, culler: &Culler, tiles: &mut Vec<SelectedTile>) {
        let camera = self.orientation.inverse() * -self.center;
        let overall = self
            .height
            .and_then(|height| height.overall_range())
            .unwrap_or(HeightRange::SMOOTH)
            .scaled(self.height_scale)
            .widened(self.relief);
        let selection = TileSelection {
            camera,
            ranges,
            culler,
            horizon: HorizonCuller::new(camera, self.radius + overall.minimum),
        };

        for root in QuadSphereTile::roots() {
            self.select_tile(root, &selection, tiles);
        }
    }

    fn select_tile(&self, tile: QuadSphereTile, selection: &TileSelection, tiles: &mut Vec<SelectedTile>) {
        let direction = tile.cube_center().normalize();
        let chord_radius = tile.chord_radius();
        let angular_radius = 2.0 * (chord_radius / 2.0).asin();

        let height = self
            .height
            .and_then(|height| height.tile_range(tile))
            .unwrap_or(HeightRange::SMOOTH)
            .scaled(self.height_scale)
            .widened(self.relief);

        if selection
            .horizon
            .is_hidden(direction, angular_radius, self.radius + height.maximum)
        {
            return;
        }

        let center = self.bounding_center(direction, height);
        let bounding_radius = self.bounding_radius(chord_radius, height);

        if !selection.culler.is_visible(center, bounding_radius) {
            return;
        }

        let nearest = self.nearest_distance(selection.camera, direction, angular_radius, height);
        let detail = selection
            .ranges
            .detail(tile.depth, nearest)
            .or((tile.depth == DEEPEST_DEPTH).then_some(DEEPEST_DEPTH as i32));

        if let Some(detail) = detail {
            tiles.push(SelectedTile { tile, detail, nearest });
            return;
        }

        for child in tile.children() {
            self.select_tile(child, selection, tiles);
        }
    }

    // Relative to the camera.
    fn bounding_center(&self, direction: DVec3, height: HeightRange) -> DVec3 {
        let middle_height = (height.minimum + height.maximum) / 2.0;
        self.center + self.orientation * (direction * (self.radius + middle_height))
    }

    fn bounding_radius(&self, chord_radius: f64, height: HeightRange) -> f64 {
        chord_radius * (self.radius + height.maximum) + (height.maximum - height.minimum) / 2.0
    }

    // The exact distance from `camera`, in the body's frame, to the shell of the cap around `direction` between its
    // lowest and highest radius: unlike a bounding sphere, the height range never widens the tile sideways.
    fn nearest_distance(&self, camera: DVec3, direction: DVec3, angular_radius: f64, height: HeightRange) -> f64 {
        let camera_radius = camera.length();
        let angle = camera.cross(direction).length().atan2(camera.dot(direction));
        let angle_outside = (angle - angular_radius).max(0.0);
        let radius =
            (camera_radius * angle_outside.cos()).clamp(self.radius + height.minimum, self.radius + height.maximum);
        let half_angle_sine = (angle_outside / 2.0).sin();

        ((camera_radius - radius).powi(2) + 4.0 * camera_radius * radius * half_angle_sine * half_angle_sine).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use ash::vk;
    use glam::DVec2;
    use xsa_core::terrain::FacePoint;

    use super::*;
    use crate::camera::Camera;

    const RADIUS: f64 = 6.0e6;
    const VERTEX_SPACING_PIXELS: f64 = 8.0;
    const EXTENT: vk::Extent2D = vk::Extent2D {
        width: 1920,
        height: 1080,
    };

    fn select(center: DVec3) -> Vec<SelectedTile> {
        select_over(center, None)
    }

    const LOWEST: f64 = -9099.0;
    const HIGHEST: f64 = 10747.0;
    const MAXIMUM_TILES: usize = 10_000;

    // Every tile spans `tile` within a body spanning LOWEST to HIGHEST.
    struct UniformHeights {
        tile: HeightRange,
    }

    impl TileHeightBounds for UniformHeights {
        fn overall_range(&self) -> Option<HeightRange> {
            Some(HeightRange {
                minimum: LOWEST,
                maximum: HIGHEST,
            })
        }

        fn tile_range(&self, _: QuadSphereTile) -> Option<HeightRange> {
            Some(self.tile)
        }
    }

    fn camera() -> Camera {
        Camera::new(FRAC_PI_2, EXTENT.width as f32 / EXTENT.height as f32, false)
    }

    fn detail_ranges() -> DetailRanges {
        let focal_length_pixels = f64::from(EXTENT.height) / 2.0 / f64::from(camera().vertical_fov() / 2.0).tan();
        DetailRanges::new(RADIUS, focal_length_pixels, VERTEX_SPACING_PIXELS)
    }

    fn select_over(center: DVec3, height: Option<&dyn TileHeightBounds>) -> Vec<SelectedTile> {
        let culler = Culler::new(&camera(), EXTENT);
        let ranges = detail_ranges();
        let sphere = QuadSphere {
            radius: RADIUS,
            height,
            relief: 0.0,
            height_scale: 1.0,
            center,
            orientation: DQuat::IDENTITY,
        };
        let mut tiles = Vec::new();
        sphere.select(&ranges, &culler, &mut tiles);
        tiles
    }

    // The camera looks along +Y: a sphere ahead, or one just below the camera to look along the ground.
    fn ahead(distance: f64) -> DVec3 {
        DVec3::new(0.0, distance, 0.0)
    }

    fn below(altitude: f64) -> DVec3 {
        DVec3::new(0.0, 0.0, -(RADIUS + altitude))
    }

    fn face_rectangle(tile: &QuadSphereTile) -> [f64; 4] {
        let size = tile.half_size();
        let left = f64::from(tile.x) * size;
        let bottom = f64::from(tile.y) * size;

        [left, bottom, left + size, bottom + size]
    }

    fn share_an_edge(first: &QuadSphereTile, second: &QuadSphereTile) -> bool {
        let [left, bottom, right, top] = face_rectangle(first);
        let [other_left, other_bottom, other_right, other_top] = face_rectangle(second);
        let overlap_x = right.min(other_right) - left.max(other_left);
        let overlap_y = top.min(other_top) - bottom.max(other_bottom);

        first.face == second.face && (overlap_x == 0.0 && overlap_y > 0.0 || overlap_y == 0.0 && overlap_x > 0.0)
    }

    #[test]
    fn a_distant_sphere_is_its_visible_roots_on_the_coarsest_grid() {
        let tiles = select(ahead(RADIUS * 200.0));

        assert!(!tiles.is_empty() && tiles.len() < 6);
        assert!(
            tiles
                .iter()
                .all(|tile| tile.tile.depth == 0 && tile.detail == COARSEST_DETAIL)
        );
    }

    #[test]
    fn the_far_side_is_never_selected() {
        let tiles = select(ahead(RADIUS * 3.0));

        assert!(tiles.iter().all(|tile| tile.tile.cube_center().normalize().y < 0.5));
    }

    #[test]
    fn tiles_get_finer_near_the_ground() {
        let tiles = select(below(100.0));
        let deepest = tiles.iter().map(|tile| tile.tile.depth).max().unwrap();

        assert!(deepest > 10);
        assert!(tiles.iter().all(|tile| tile.tile.cube_center().normalize().z > 0.9));
    }

    // Global bounds would put the camera inside every nearby tile's bounding sphere, splitting them all to the
    // deepest level.
    #[test]
    fn tiles_stay_few_just_above_terrain() {
        let ground = 812.0;
        let height = UniformHeights {
            tile: HeightRange {
                minimum: ground,
                maximum: ground,
            },
        };
        let tiles = select_over(below(ground + 100.0), Some(&height));

        assert!(tiles.len() < MAXIMUM_TILES, "{} tiles", tiles.len());
        assert!(tiles.iter().map(|tile| tile.tile.depth).max().unwrap() > 10);
    }

    // No tile's bounds are tight: each spans the whole body's heights.
    #[test]
    fn tiles_stay_few_inside_rough_terrain() {
        let height = UniformHeights {
            tile: HeightRange {
                minimum: LOWEST,
                maximum: HIGHEST,
            },
        };
        let tiles = select_over(below(0.0), Some(&height));

        assert!(tiles.len() < MAXIMUM_TILES, "{} tiles", tiles.len());

        for first in &tiles {
            for second in tiles.iter().filter(|second| share_an_edge(&first.tile, &second.tile)) {
                assert!(
                    (first.detail - second.detail).abs() <= 1,
                    "{first:?} next to {second:?}"
                );
            }
        }
    }

    // Where a tile meets a coarser neighbor, its corners on their shared edge are fully morphed and the neighbor's
    // are not morphing yet, so both draw the coarser level's grid there and no crack opens.
    #[test]
    fn neighbors_meet_on_the_coarser_level() {
        let mut checked = 0;

        for altitude in [2.0, 100.0, 1.0e4, 1.0e6] {
            let center = below(altitude);
            let tiles = select(center);
            let ranges = detail_ranges();

            for fine in &tiles {
                for coarse in tiles.iter().filter(|coarse| coarse.detail == fine.detail - 1) {
                    if !share_an_edge(&fine.tile, &coarse.tile) {
                        continue;
                    }

                    let [left, bottom, right, top] = face_rectangle(&coarse.tile);
                    let [fine_left, fine_bottom, fine_right, fine_top] = face_rectangle(&fine.tile);
                    let corners = [
                        DVec2::new(fine_left, fine_bottom),
                        DVec2::new(fine_right, fine_bottom),
                        DVec2::new(fine_left, fine_top),
                        DVec2::new(fine_right, fine_top),
                    ];

                    for corner in corners {
                        if corner.x < left || corner.x > right || corner.y < bottom || corner.y > top {
                            continue;
                        }

                        let point = FacePoint {
                            face: fine.tile.face,
                            coordinates: corner * 2.0 - 1.0,
                        };
                        let distance = (center + point.direction() * RADIUS).length();

                        assert!(distance >= ranges.morph_end(fine.detail), "{fine:?} next to {coarse:?}");
                        assert!(
                            distance <= ranges.morph_start(coarse.detail),
                            "{fine:?} next to {coarse:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }

        assert!(checked > 100, "{checked} corners checked");
    }

    #[test]
    fn neighbors_differ_by_at_most_one_level() {
        for altitude in [100.0, 1.0e4, 1.0e6] {
            let tiles = select(below(altitude));

            for first in &tiles {
                for second in tiles.iter().filter(|second| share_an_edge(&first.tile, &second.tile)) {
                    assert!(
                        (first.detail - second.detail).abs() <= 1,
                        "{first:?} next to {second:?}"
                    );
                }
            }
        }
    }
}
