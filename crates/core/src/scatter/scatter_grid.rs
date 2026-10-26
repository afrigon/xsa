use std::f64::consts::PI;

use glam::DVec3;

use crate::terrain::QuadSphereTile;

// A page holds 2^PAGE_LEVELS cells along each side.
pub const PAGE_LEVELS: u32 = 4;

// The cells a scatter is placed in: quad sphere tiles at the shallowest depth whose largest cell, at a face's center, is
// no wider than the spacing, grouped into pages of 2^PAGE_LEVELS by 2^PAGE_LEVELS cells, generated and drawn together.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScatterGrid {
    pub cell_depth: u32,
}

impl ScatterGrid {
    // At a face's center a cell spans π / 2^(depth + 1) radians.
    pub fn new(radius: f64, spacing: f64) -> ScatterGrid {
        let cell_depth = (PI * radius / (2.0 * spacing))
            .log2()
            .ceil()
            .max(f64::from(PAGE_LEVELS));

        ScatterGrid {
            cell_depth: cell_depth as u32,
        }
    }

    pub fn page_depth(self) -> u32 {
        self.cell_depth - PAGE_LEVELS
    }

    // Every page reaching within `angle` radians of `direction`, a unit vector in the body's frame.
    pub fn pages_within(self, direction: DVec3, angle: f64) -> Vec<QuadSphereTile> {
        let mut pages = Vec::new();

        for root in QuadSphereTile::roots() {
            self.collect_pages(root, direction, angle, &mut pages);
        }

        pages
    }

    fn collect_pages(self, tile: QuadSphereTile, direction: DVec3, angle: f64, pages: &mut Vec<QuadSphereTile>) {
        let center = tile.cube_center().normalize();
        let tile_angle = 2.0 * (tile.chord_radius() / 2.0).asin();

        if center.angle_between(direction) > angle + tile_angle {
            return;
        }

        if tile.depth == self.page_depth() {
            pages.push(tile);
            return;
        }

        for child in tile.children() {
            self.collect_pages(child, direction, angle, pages);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LUNA_RADIUS: f64 = 1_737_400.0;

    #[test]
    fn cells_are_no_wider_than_the_spacing() {
        for spacing in [2.0, 40.0, 1000.0] {
            let grid = ScatterGrid::new(LUNA_RADIUS, spacing);
            let widest = PI * LUNA_RADIUS / f64::from(grid.cell_depth + 1).exp2();

            assert!(widest <= spacing);
            assert!(2.0 * widest > spacing, "a shallower depth would do for {spacing} m");
        }
    }

    #[test]
    fn pages_cover_the_cap_around_a_direction() {
        let grid = ScatterGrid::new(LUNA_RADIUS, 40.0);
        let direction = DVec3::new(0.3, -0.8, 0.52).normalize();
        let angle = 3000.0 / LUNA_RADIUS;
        let pages = grid.pages_within(direction, angle);

        assert!(!pages.is_empty());
        assert!(pages.iter().all(|page| page.depth == grid.page_depth()));

        for step in 0..16 {
            let turn = f64::from(step) / 16.0 * 2.0 * PI;
            let side = direction.any_orthonormal_vector();
            let toward = side * turn.cos() + direction.cross(side) * turn.sin();
            let point = (direction + toward * angle * 0.99).normalize();
            let point = crate::terrain::FacePoint::from_direction(point);
            let covered = pages.iter().any(|page| {
                let center = page.face_center();
                page.face == point.face && (point.coordinates - center).abs().max_element() <= page.half_size()
            });

            assert!(covered, "no page covers the cap's edge at turn {turn}");
        }
    }
}
