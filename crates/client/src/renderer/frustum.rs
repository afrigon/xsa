use glam::{DMat4, DVec3, DVec4};

const LEFT_RIGHT_ROW: usize = 0;
const BOTTOM_TOP_ROW: usize = 1;
const DEPTH_ROW: usize = 2;
const W_ROW: usize = 3;

// Planes face inward: a point is inside when its signed distance to every plane is positive. Reverse-Z
// infinite projections have no far plane.
pub(in crate::renderer) struct Frustum {
    planes: [DVec4; 5],
}

impl Frustum {
    pub fn from_view_projection(view_projection: DMat4) -> Self {
        let w = view_projection.row(W_ROW);
        let horizontal = view_projection.row(LEFT_RIGHT_ROW);
        let vertical = view_projection.row(BOTTOM_TOP_ROW);
        let depth = view_projection.row(DEPTH_ROW);

        Self {
            planes: [w + horizontal, w - horizontal, w + vertical, w - vertical, w - depth]
                .map(|plane| plane / plane.truncate().length()),
        }
    }

    pub fn contains_sphere(&self, center: DVec3, radius: f64) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.truncate().dot(center) + plane.w >= -radius)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;
    use crate::camera::Camera;

    const ASPECT_RATIO: f32 = 16.0 / 9.0;
    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;

    fn frustum() -> Frustum {
        let camera = Camera::new(FRAC_PI_2, ASPECT_RATIO);
        let view_projection = camera.clip_from_view() * camera.view_rotation();
        Frustum::from_view_projection(view_projection.as_dmat4())
    }

    #[test]
    fn sphere_ahead_is_inside() {
        assert!(frustum().contains_sphere(DVec3::new(0.0, 100.0, 0.0), 1.0));
    }

    #[test]
    fn sphere_behind_is_outside() {
        assert!(!frustum().contains_sphere(DVec3::new(0.0, -100.0, 0.0), 1.0));
    }

    #[test]
    fn sphere_beyond_the_side_edge_is_outside() {
        assert!(!frustum().contains_sphere(DVec3::new(200.0, 100.0, 0.0), 1.0));
    }

    #[test]
    fn sphere_overlapping_the_side_edge_is_inside() {
        assert!(frustum().contains_sphere(DVec3::new(200.0, 100.0, 0.0), 80.0));
    }

    #[test]
    fn sphere_surrounding_the_camera_is_inside() {
        assert!(frustum().contains_sphere(DVec3::new(0.0, -10.0, 0.0), 20.0));
    }

    #[test]
    fn sphere_at_interplanetary_distance_is_inside() {
        assert!(frustum().contains_sphere(DVec3::new(0.0, 30.0 * ASTRONOMICAL_UNIT, 0.0), 1.0e6));
    }
}
