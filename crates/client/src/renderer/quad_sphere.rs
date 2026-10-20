mod detail_ranges;
mod height_range;
mod horizon_culler;
mod quad_sphere_tile;
mod selected_tile;
mod tile_height_bounds;

pub(in crate::renderer) use detail_ranges::{COARSEST_DETAIL, DetailRanges};
pub(in crate::renderer) use height_range::HeightRange;
pub(in crate::renderer) use horizon_culler::HorizonCuller;
pub(in crate::renderer) use quad_sphere_tile::QuadSphereTile;
pub(in crate::renderer) use selected_tile::SelectedTile;
pub(in crate::renderer) use tile_height_bounds::TileHeightBounds;

use glam::{DQuat, DVec3};

use super::culler::Culler;

// A tile's grid points sit at offsets from its center that f32 holds exactly down to this depth.
pub(in crate::renderer) const DEEPEST_DEPTH: u32 = 22;

// A sphere whose cube faces split into quadtrees of tiles, finer where the camera is closer.
pub(in crate::renderer) struct QuadSphere<'a> {
    pub radius: f64,
    // None for a smooth sphere.
    pub height: Option<&'a dyn TileHeightBounds>,
    // Relative to the camera.
    pub center: DVec3,
    pub orientation: DQuat,
}

impl QuadSphere<'_> {
    pub fn select(&self, ranges: &DetailRanges, culler: &Culler, tiles: &mut Vec<SelectedTile>) {
        let camera = self.orientation.inverse() * -self.center;
        let lowest = self
            .height
            .and_then(|height| height.overall_range())
            .map_or(0.0, |range| range.minimum);
        let horizon = HorizonCuller::new(camera, self.radius + lowest);

        for root in QuadSphereTile::roots() {
            self.select_tile(root, camera, ranges, culler, &horizon, tiles);
        }
    }

    fn select_tile(
        &self,
        tile: QuadSphereTile,
        camera: DVec3,
        ranges: &DetailRanges,
        culler: &Culler,
        horizon: &HorizonCuller,
        tiles: &mut Vec<SelectedTile>,
    ) {
        let direction = tile.cube_center().normalize();
        let chord_radius = tile.chord_radius();
        let angular_radius = 2.0 * (chord_radius / 2.0).asin();

        let height = self
            .height
            .and_then(|height| height.tile_range(tile))
            .unwrap_or(HeightRange::SMOOTH);

        if horizon.is_hidden(direction, angular_radius, self.radius + height.maximum) {
            return;
        }

        let center = self.bounding_center(direction, height);
        let bounding_radius = self.bounding_radius(chord_radius, height);

        if !culler.is_visible(center, bounding_radius) {
            return;
        }

        let nearest = self.nearest_distance(camera, direction, angular_radius, height);
        let detail = ranges
            .detail(tile.depth, nearest)
            .or((tile.depth == DEEPEST_DEPTH).then_some(DEEPEST_DEPTH as i32));

        if let Some(detail) = detail {
            tiles.push(SelectedTile {
                tile,
                detail,
                nearest,
                farthest: center.length() + bounding_radius,
            });
            return;
        }

        for child in tile.children() {
            self.select_tile(child, camera, ranges, culler, horizon, tiles);
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

    fn select_over(center: DVec3, height: Option<&dyn TileHeightBounds>) -> Vec<SelectedTile> {
        let camera = Camera::new(FRAC_PI_2, EXTENT.width as f32 / EXTENT.height as f32);
        let culler = Culler::new(&camera, EXTENT);
        let focal_length_pixels = f64::from(EXTENT.height) / 2.0 / f64::from(camera.vertical_fov() / 2.0).tan();
        let ranges = DetailRanges::new(RADIUS, focal_length_pixels, VERTEX_SPACING_PIXELS);
        let sphere = QuadSphere {
            radius: RADIUS,
            height,
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
