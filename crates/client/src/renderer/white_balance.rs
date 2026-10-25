use std::time::Instant;

use glam::{Mat3, Vec3, Vec4};

use super::auto_exposure::EXPOSURE_TRANSITION_HALF_LIFE_SECONDS;

// Relative luminance of linear sRGB.
pub(super) const LUMINANCE_WEIGHTS: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);
// CIE XYZ of linear sRGB, rows as published (IEC 61966-2-1).
const XYZ_FROM_SRGB_ROWS: [f32; 9] = [0.4124, 0.3576, 0.1805, 0.2126, 0.7152, 0.0722, 0.0193, 0.1192, 0.9505];
// Bradford cone responses of CIE XYZ, rows as published.
const BRADFORD_ROWS: [f32; 9] = [
    0.8951, 0.2664, -0.1614, -0.7502, 1.7135, 0.0367, 0.0389, -0.0685, 1.0296,
];

// The camera's white balance: like an eye, it takes the light of the star lighting the scene as white. A Bradford
// chromatic adaptation maps that light to the display's white, so a gray surface in it renders gray.
pub struct WhiteBalance {
    white: Vec3,
    last_update: Option<Instant>,
    duration_scale: f32,
}

impl WhiteBalance {
    pub fn new() -> Self {
        Self {
            white: Vec3::ONE,
            last_update: None,
            duration_scale: 1.0,
        }
    }

    pub fn configure(&mut self, duration_scale: f32) {
        self.duration_scale = duration_scale;
    }

    // The color of the light to adapt to, in linear sRGB with a luminance of 1.
    pub fn update(&mut self, light: Vec3, now: Instant) {
        let elapsed = self.last_update.map(|last| now.duration_since(last).as_secs_f32());
        self.last_update = Some(now);
        self.white = match elapsed {
            Some(seconds) => self.transition(self.white, light, seconds),
            None => light,
        };
    }

    // Rows of the linear sRGB matrix taking the adapted white to the display's white; disabled, the identity, while
    // the adaptation keeps following the light so enabling it again takes effect at once.
    pub fn rows(&self, enabled: bool) -> [Vec4; 3] {
        let white = if enabled { self.white } else { Vec3::ONE };
        let adaptation = WhiteBalance::adaptation(white);
        let rows = adaptation.transpose();
        [
            rows.x_axis.extend(0.0),
            rows.y_axis.extend(0.0),
            rows.z_axis.extend(0.0),
        ]
    }

    fn adaptation(white: Vec3) -> Mat3 {
        let xyz_from_srgb = Mat3::from_cols_array(&XYZ_FROM_SRGB_ROWS).transpose();
        let bradford = Mat3::from_cols_array(&BRADFORD_ROWS).transpose();
        let cone_scale = (bradford * xyz_from_srgb * Vec3::ONE) / (bradford * xyz_from_srgb * white);
        xyz_from_srgb.inverse() * bradford.inverse() * Mat3::from_diagonal(cone_scale) * bradford * xyz_from_srgb
    }

    fn transition(&self, current: Vec3, target: Vec3, seconds: f32) -> Vec3 {
        let half_life = EXPOSURE_TRANSITION_HALF_LIFE_SECONDS * self.duration_scale;

        if half_life <= 0.0 {
            return target;
        }

        current + (target - current) * (1.0 - 0.5_f32.powf(seconds / half_life))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A 5772 K blackbody, the Sun, in linear sRGB with a luminance of 1.
    const SUNLIGHT: Vec3 = Vec3::new(1.113, 0.975, 0.913);

    fn apply(rows: [Vec4; 3], color: Vec3) -> Vec3 {
        Vec3::new(
            rows[0].truncate().dot(color),
            rows[1].truncate().dot(color),
            rows[2].truncate().dot(color),
        )
    }

    #[test]
    fn the_display_white_needs_no_correction() {
        let rows = WhiteBalance::new().rows(true);
        let color = Vec3::new(0.3, 0.5, 0.7);
        assert!((apply(rows, color) - color).abs().max_element() < 1e-4);
    }

    #[test]
    fn the_adapted_light_becomes_white() {
        let mut white_balance = WhiteBalance::new();
        white_balance.update(SUNLIGHT, Instant::now());
        let white = apply(white_balance.rows(true), SUNLIGHT);
        assert!((white - Vec3::ONE).abs().max_element() < 1e-3, "{white}");
    }

    #[test]
    fn a_gray_surface_in_that_light_renders_gray() {
        let mut white_balance = WhiteBalance::new();
        white_balance.update(SUNLIGHT, Instant::now());
        let gray = apply(white_balance.rows(true), SUNLIGHT * 0.18);
        assert!((gray - Vec3::splat(0.18)).abs().max_element() < 1e-3, "{gray}");
    }

    #[test]
    fn disabled_it_leaves_colors_alone() {
        let mut white_balance = WhiteBalance::new();
        white_balance.update(SUNLIGHT, Instant::now());
        let color = Vec3::new(0.3, 0.5, 0.7);
        assert!((apply(white_balance.rows(false), color) - color).abs().max_element() < 1e-4);
    }

    #[test]
    fn luminance_is_kept() {
        let mut white_balance = WhiteBalance::new();
        white_balance.update(SUNLIGHT, Instant::now());
        let color = Vec3::new(0.4, 0.2, 0.1);
        let balanced = apply(white_balance.rows(true), color);
        assert!((balanced.dot(LUMINANCE_WEIGHTS) - color.dot(LUMINANCE_WEIGHTS)).abs() < 0.01);
    }
}
