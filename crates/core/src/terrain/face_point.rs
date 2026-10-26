use std::f64::consts::FRAC_PI_4;

use glam::{DVec2, DVec3};
use resource_loader::terrain::{PageAddress, PageKey};
use resource_loader::utils::CubeFace;

// A direction on a body as a point of its cube map: the face it falls on and its coordinates there, -1 to 1 along
// the face's columns and rows. Faces are equal-angle: a coordinate s lies at the angle π/4 · s from the face's center.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FacePoint {
    pub face: CubeFace,
    pub coordinates: DVec2,
}

impl FacePoint {
    // `direction` is a unit vector in the body's frame.
    pub fn from_direction(direction: DVec3) -> FacePoint {
        let magnitude = direction.abs();
        let axis = if magnitude.x >= magnitude.y && magnitude.x >= magnitude.z {
            DVec3::X * direction.x.signum()
        } else if magnitude.y >= magnitude.z {
            DVec3::Y * direction.y.signum()
        } else {
            DVec3::Z * direction.z.signum()
        };
        let face = CubeFace::from_normal(axis.as_ivec3()).expect("a signed unit axis is a face's normal");

        FacePoint::on_face(face, direction)
    }

    // The point of `face` toward `direction`, which must lie in front of it; past its edges, coordinates exceed 1.
    pub fn on_face(face: CubeFace, direction: DVec3) -> FacePoint {
        let depth = direction.dot(face.normal().as_dvec3());
        let plane = DVec2::new(
            direction.dot(face.column_axis().as_dvec3()),
            direction.dot(face.row_axis().as_dvec3()),
        ) / depth;

        FacePoint {
            face,
            coordinates: DVec2::new(plane.x.atan(), plane.y.atan()) / FRAC_PI_4,
        }
    }

    // A unit vector in the body's frame.
    pub fn direction(self) -> DVec3 {
        let plane = (self.coordinates * FRAC_PI_4).to_array().map(f64::tan);

        (self.face.normal().as_dvec3()
            + plane[0] * self.face.column_axis().as_dvec3()
            + plane[1] * self.face.row_axis().as_dvec3())
        .normalize()
    }

    // Its position on a face `extent` texels wide, texel centers at whole coordinates.
    pub fn texel_position(self, extent: u32) -> DVec2 {
        (self.coordinates + 1.0) / 2.0 * f64::from(extent) - 0.5
    }

    // The page at `level` whose own area holds the point.
    pub fn page(self, level: u32) -> PageAddress {
        let pages_per_side = 1u32 << level;
        let page = ((self.coordinates + 1.0) / 2.0 * f64::from(pages_per_side))
            .floor()
            .clamp(DVec2::ZERO, DVec2::splat(f64::from(pages_per_side - 1)));

        PageAddress {
            face: self.face,
            level,
            key: PageKey::from_coordinates(page.x as u32, page.y as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use resource_loader::terrain::PAGE_INTERIOR;

    use super::*;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn a_face_center_is_its_normal() {
        for face in CubeFace::ALL {
            let point = FacePoint::from_direction(face.normal().as_dvec3());

            assert_eq!(point.face, face);
            assert!(point.coordinates.length() < TOLERANCE);
        }
    }

    #[test]
    fn directions_round_trip() {
        let direction = DVec3::new(0.3, -0.8, 0.52).normalize();
        let point = FacePoint::from_direction(direction);

        assert_eq!(point.face, CubeFace::NegativeY);
        assert!(point.direction().distance(direction) < TOLERANCE);
    }

    // Equal-angle: a quarter of the way from the center to an edge is a quarter of the 45° between them.
    #[test]
    fn coordinates_are_proportional_to_angles() {
        let angle = FRAC_PI_4 / 4.0;
        let direction = DVec3::new(angle.cos(), 0.0, -angle.sin());
        let point = FacePoint::from_direction(direction);

        assert_eq!(point.face, CubeFace::PositiveX);
        assert!((point.coordinates.x - 0.25).abs() < TOLERANCE);
        assert!(point.coordinates.y.abs() < TOLERANCE);
    }

    #[test]
    fn pages_follow_columns_and_rows() {
        let point = FacePoint {
            face: CubeFace::PositiveZ,
            coordinates: DVec2::new(-0.9, 0.6),
        };
        let page = point.page(2);

        assert_eq!((page.key.x(), page.key.y()), (0, 3));
        let expected = DVec2::new(0.05 * 128.0 - 0.5, 0.8 * 128.0 - 0.5);
        assert!(point.texel_position(PAGE_INTERIOR).distance(expected) < TOLERANCE);
    }
}
