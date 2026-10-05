use glam::{DMat3, DQuat, DVec3};

use super::Camera;
use super::orbit::OrbitTarget;

const HORIZONTAL_FIELD_OF_VIEW_DEGREES: f64 = 40.0;
const AZIMUTH_FROM_STAR_DEGREES: f64 = 35.0;
const ELEVATION_ABOVE_EQUATOR_DEGREES: f64 = 20.0;
// Sizes on screen in half-widths of the screen, so the framing holds at any aspect ratio.
const RADIUS_IN_HALF_WIDTHS: f64 = 0.73;
const RIGHTWARD_OFFSET_IN_HALF_WIDTHS: f64 = 0.5;
const DOWNWARD_OFFSET_IN_HALF_WIDTHS: f64 = 0.39;

// Frames a body large in the bottom right of the screen, its pole up so it turns about the screen's vertical, seen
// from its lit side a little off the star's direction so the terminator shows.
pub struct MenuCamera;

impl MenuCamera {
    pub fn update(&self, camera: &mut Camera, target: OrbitTarget, star_position: Option<DVec3>, pole: DVec3) {
        let star_direction = star_position.map_or(DVec3::ZERO, |star| star - target.position);
        let equatorial = (star_direction - pole * star_direction.dot(pole)).normalize_or(pole.any_orthonormal_vector());
        let heading = DQuat::from_axis_angle(pole, AZIMUTH_FROM_STAR_DEGREES.to_radians()) * equatorial;
        let elevation = ELEVATION_ABOVE_EQUATOR_DEGREES.to_radians();
        let view_direction = heading * elevation.cos() + pole * elevation.sin();

        let half_field_of_view_tangent = (HORIZONTAL_FIELD_OF_VIEW_DEGREES.to_radians() / 2.0).tan();
        let angular_radius = (RADIUS_IN_HALF_WIDTHS * half_field_of_view_tangent).atan();
        let distance = target.radius / angular_radius.sin();
        let yaw = (RIGHTWARD_OFFSET_IN_HALF_WIDTHS * half_field_of_view_tangent).atan();
        let pitch = (DOWNWARD_OFFSET_IN_HALF_WIDTHS * half_field_of_view_tangent).atan();

        let forward = -view_direction;
        let right = forward.cross(pole).normalize();
        let up = right.cross(forward);
        let looking_at_body = DQuat::from_mat3(&DMat3::from_cols(right, forward, up));

        camera.horizontal_fov = HORIZONTAL_FIELD_OF_VIEW_DEGREES.to_radians() as f32;
        camera.position = target.position + view_direction * distance;
        camera.orientation = looking_at_body * DQuat::from_rotation_z(yaw) * DQuat::from_rotation_x(pitch);
    }
}
