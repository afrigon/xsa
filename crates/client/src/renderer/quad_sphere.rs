mod cube_face;
mod detail_ranges;
mod horizon_culler;
mod quad_sphere_tile;
mod selected_tile;

pub(in crate::renderer) use cube_face::CubeFace;
pub(in crate::renderer) use detail_ranges::{COARSEST_DETAIL, DetailRanges};
pub(in crate::renderer) use horizon_culler::HorizonCuller;
pub(in crate::renderer) use quad_sphere_tile::QuadSphereTile;
pub(in crate::renderer) use selected_tile::SelectedTile;

use glam::{DQuat, DVec3};

use super::culler::Culler;

// Tile centers are sent to the GPU as cube coordinates in f32, which hold them exactly down to this depth.
pub(in crate::renderer) const DEEPEST_DEPTH: u32 = 22;

// A sphere whose cube faces split into quadtrees of tiles, finer where the camera is closer.
pub(in crate::renderer) struct QuadSphere {
    pub radius: f64,
    // Relative to the radius, signed.
    pub minimum_height: f64,
    pub maximum_height: f64,
    // Relative to the camera.
    pub center: DVec3,
    pub orientation: DQuat,
}

impl QuadSphere {
    pub fn select(&self, ranges: &DetailRanges, culler: &Culler, tiles: &mut Vec<SelectedTile>) {
        let camera = self.orientation.inverse() * -self.center;
        let horizon = HorizonCuller::new(
            camera,
            self.radius + self.minimum_height,
            self.radius + self.maximum_height,
        );

        for root in QuadSphereTile::roots() {
            self.select_tile(root, ranges, culler, &horizon, tiles);
        }
    }

    fn select_tile(
        &self,
        tile: QuadSphereTile,
        ranges: &DetailRanges,
        culler: &Culler,
        horizon: &HorizonCuller,
        tiles: &mut Vec<SelectedTile>,
    ) {
        let direction = tile.cube_center().normalize();
        let chord_radius = tile.chord_radius();
        let angular_radius = 2.0 * (chord_radius / 2.0).asin();

        if horizon.is_hidden(direction, angular_radius) {
            return;
        }

        let middle_height = (self.minimum_height + self.maximum_height) / 2.0;
        let half_height_range = (self.maximum_height - self.minimum_height) / 2.0;
        let center = self.center + self.orientation * (direction * (self.radius + middle_height));
        let bounding_radius = chord_radius * (self.radius + self.maximum_height) + half_height_range;

        if !culler.is_visible(center, bounding_radius) {
            return;
        }

        let nearest = center.length() - bounding_radius;
        let detail = ranges
            .detail(tile.depth, nearest)
            .or((tile.depth == DEEPEST_DEPTH).then_some(DEEPEST_DEPTH as i32));

        if let Some(detail) = detail {
            tiles.push(SelectedTile { tile, detail });
            return;
        }

        for child in tile.children() {
            self.select_tile(child, ranges, culler, horizon, tiles);
        }
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
        let camera = Camera::new(FRAC_PI_2, EXTENT.width as f32 / EXTENT.height as f32);
        let culler = Culler::new(&camera, EXTENT);
        let focal_length_pixels = f64::from(EXTENT.height) / 2.0 / f64::from(camera.vertical_fov() / 2.0).tan();
        let ranges = DetailRanges::new(RADIUS, focal_length_pixels, VERTEX_SPACING_PIXELS);
        let sphere = QuadSphere {
            radius: RADIUS,
            minimum_height: 0.0,
            maximum_height: 0.0,
            center,
            orientation: DQuat::IDENTITY,
        };
        let mut tiles = Vec::new();
        sphere.select(&ranges, &culler, &mut tiles);
        tiles
    }

    // The camera looks along +Y: a sphere ahead, or one just below the camera to look along its surface.
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
    fn tiles_get_finer_near_the_surface() {
        let tiles = select(below(100.0));
        let deepest = tiles.iter().map(|tile| tile.tile.depth).max().unwrap();

        assert!(deepest > 10);
        assert!(tiles.iter().all(|tile| tile.tile.cube_center().normalize().z > 0.9));
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
