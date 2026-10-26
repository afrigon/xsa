use glam::{DVec2, DVec3};
use resource_loader::utils::CubeFace;
use xsa_core::terrain::FacePoint;

const CHILDREN_PER_SIDE: u32 = 2;

// A tile's identity: stable from frame to frame, so data computed for a tile can be cached under it. Tiles split their
// face evenly in its equal-angle coordinates, columns `x` and rows `y` along the face's, as the terrain's cube maps
// split their faces into texels.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) struct QuadSphereTile {
    pub face: CubeFace,
    pub depth: u32,
    pub x: u32,
    pub y: u32,
}

impl QuadSphereTile {
    pub fn roots() -> impl Iterator<Item = QuadSphereTile> {
        CubeFace::ALL.into_iter().map(|face| QuadSphereTile {
            face,
            depth: 0,
            x: 0,
            y: 0,
        })
    }

    pub fn children(self) -> [QuadSphereTile; 4] {
        let child = |column, row| QuadSphereTile {
            face: self.face,
            depth: self.depth + 1,
            x: self.x * CHILDREN_PER_SIDE + column,
            y: self.y * CHILDREN_PER_SIDE + row,
        };

        [child(0, 0), child(1, 0), child(0, 1), child(1, 1)]
    }

    // Half the tile's side, in its face's coordinates.
    pub fn half_size(self) -> f64 {
        f64::from(self.depth).exp2().recip()
    }

    // In its face's coordinates, -1 to 1 along the face's columns and rows.
    pub fn face_center(self) -> DVec2 {
        (DVec2::new(f64::from(2 * self.x + 1), f64::from(2 * self.y + 1))) * self.half_size() - 1.0
    }

    // On the cube spanning -1 to 1.
    pub fn cube_center(self) -> DVec3 {
        self.cube_point(self.face_center())
    }

    pub fn column_axis(self) -> DVec3 {
        self.face.column_axis().as_dvec3()
    }

    pub fn row_axis(self) -> DVec3 {
        self.face.row_axis().as_dvec3()
    }

    pub fn cube_corners(self) -> [DVec3; 4] {
        let center = self.face_center();
        let half_size = self.half_size();

        [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
            .map(|(column, row)| self.cube_point(center + DVec2::new(column, row) * half_size))
    }

    // The largest distance from the tile's center to any of its points, on the unit sphere.
    pub fn chord_radius(self) -> f64 {
        let center = self.cube_center().normalize();

        self.cube_corners()
            .iter()
            .map(|corner| corner.normalize().distance(center))
            .fold(0.0, f64::max)
    }

    fn cube_point(self, coordinates: DVec2) -> DVec3 {
        let direction = FacePoint {
            face: self.face,
            coordinates,
        }
        .direction();

        direction / direction.dot(self.face.normal().as_dvec3())
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::SQRT_2;

    use super::*;

    #[test]
    fn children_split_the_parent_into_quarters() {
        let parent = QuadSphereTile {
            face: CubeFace::PositiveY,
            depth: 3,
            x: 5,
            y: 1,
        };
        let children = parent.children();
        let average = children.iter().map(|child| child.face_center()).sum::<DVec2>() / 4.0;

        assert!(average.distance(parent.face_center()) < 1e-12);
        assert!(
            children
                .iter()
                .all(|child| child.half_size() == parent.half_size() / 2.0)
        );
    }

    #[test]
    fn roots_cover_the_six_faces() {
        let centers: Vec<DVec3> = QuadSphereTile::roots().map(QuadSphereTile::cube_center).collect();

        assert_eq!(centers.len(), 6);
        assert!(centers.iter().all(|center| center.length() == 1.0));
    }

    #[test]
    fn faces_meet_along_their_edges() {
        for root in QuadSphereTile::roots() {
            let corners = root.cube_corners();

            for corner in [corners[0], corners[3]] {
                let faces_at_corner = CubeFace::ALL
                    .iter()
                    .filter(|other| (corner.dot(other.normal().as_dvec3()) - 1.0).abs() < 1e-12)
                    .count();
                assert_eq!(faces_at_corner, 3);
            }
        }
    }

    // DetailRanges relies on this bound to keep neighboring tiles within one level of each other.
    #[test]
    fn chord_radius_is_bounded_by_the_depth() {
        for depth in 1..=6 {
            let tiles_per_side = 1 << depth;

            for face in CubeFace::ALL {
                for x in 0..tiles_per_side {
                    for y in 0..tiles_per_side {
                        let tile = QuadSphereTile { face, depth, x, y };
                        assert!(tile.chord_radius() <= SQRT_2 * tile.half_size());
                    }
                }
            }
        }
    }
}
