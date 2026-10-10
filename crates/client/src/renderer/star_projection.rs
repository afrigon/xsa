use ash::vk;
use glam::{DMat4, DVec2, DVec3, Vec3};

use super::gpu_data::ProjectedStarData;
use crate::camera::Camera;

pub(in crate::renderer) struct StarProjection {
    view_projection: DMat4,
    near: f64,
    extent: DVec2,
    focal_length_pixels: f64,
}

impl StarProjection {
    pub fn new(camera: &Camera, extent: vk::Extent2D) -> StarProjection {
        let clip_from_view = camera.clip_from_view().as_dmat4();
        let height = f64::from(extent.height);

        StarProjection {
            view_projection: clip_from_view * camera.view_rotation().as_dmat4(),
            near: f64::from(camera.near),
            extent: DVec2::new(f64::from(extent.width), height),
            // Negative in Vulkan projections, which flip Y.
            focal_length_pixels: clip_from_view.y_axis.y.abs() * height / 2.0,
        }
    }

    pub fn focal_length_pixels(&self) -> f32 {
        self.focal_length_pixels as f32
    }

    // The star's own sphere writes depth: only samples more than one stellar radius in front of its surface hide
    // it, which keeps depth precision from hiding the star behind itself. Reverse-Z infinite depth is near / depth.
    fn occlusion_depth(&self, view_depth: f64, radius: f64) -> f32 {
        let occluder_depth = view_depth - 2.0 * radius;

        if occluder_depth <= 0.0 {
            return f32::MAX;
        }

        (self.near / occluder_depth) as f32
    }

    // `position` is relative to the camera; `None` when the star is behind the camera or surrounds it.
    pub fn project(&self, position: DVec3, radius: f64, intensity: Vec3) -> Option<ProjectedStarData> {
        let distance = position.length();
        let clip = self.view_projection * position.extend(1.0);

        if clip.w <= 0.0 || distance <= radius {
            return None;
        }

        let normalized = clip.truncate().truncate() / clip.w;
        let angular_radius = (radius / distance).asin();

        Some(ProjectedStarData {
            screen_position: ((normalized * 0.5 + 0.5) * self.extent).as_vec2(),
            pixel_radius: (angular_radius.tan() * self.focal_length_pixels) as f32,
            occlusion_depth: self.occlusion_depth(clip.w, radius),
            illuminance: intensity / (distance * distance) as f32,
            angular_radius: angular_radius as f32,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    const EXTENT: vk::Extent2D = vk::Extent2D {
        width: 1920,
        height: 1080,
    };
    const SUN_RADIUS: f64 = 6.957e8;
    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;

    fn projection() -> StarProjection {
        let aspect_ratio = EXTENT.width as f32 / EXTENT.height as f32;

        StarProjection::new(&Camera::new(FRAC_PI_2, aspect_ratio), EXTENT)
    }

    #[test]
    fn a_star_straight_ahead_lands_on_the_screen_center() {
        let star = projection()
            .project(DVec3::new(0.0, ASTRONOMICAL_UNIT, 0.0), SUN_RADIUS, Vec3::ONE)
            .unwrap();

        assert!((star.screen_position.x - 960.0).abs() < 0.5, "{}", star.screen_position);
        assert!((star.screen_position.y - 540.0).abs() < 0.5, "{}", star.screen_position);
    }

    #[test]
    fn the_sun_from_earth_is_about_a_quarter_degree_wide() {
        let star = projection()
            .project(DVec3::new(0.0, ASTRONOMICAL_UNIT, 0.0), SUN_RADIUS, Vec3::ONE)
            .unwrap();
        let degrees = star.angular_radius.to_degrees();

        assert!((0.26..0.27).contains(&degrees), "{degrees}°");
    }

    #[test]
    fn a_star_behind_the_camera_is_not_projected() {
        assert!(
            projection()
                .project(DVec3::new(0.0, -ASTRONOMICAL_UNIT, 0.0), SUN_RADIUS, Vec3::ONE)
                .is_none()
        );
    }
}
