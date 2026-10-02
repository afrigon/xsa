use std::f64::consts::PI;

use glam::{DVec3, Vec3};
use xsa_core::packs::data::Star;

const PLANCK: f64 = 6.626_070_15e-34;
const SPEED_OF_LIGHT: f64 = 299_792_458.0;
const BOLTZMANN: f64 = 1.380_649e-23;
const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;
const MAXIMUM_LUMINOUS_EFFICACY: f64 = 683.0;
const VISIBLE_START_NANOMETERS: f64 = 360.0;
const VISIBLE_END_NANOMETERS: f64 = 830.0;
const INTEGRATION_STEP_NANOMETERS: f64 = 1.0;

pub struct StarLight {
    // Linear sRGB with a luminance of 1.
    pub color: Vec3,
    // Candela: illuminance at a distance d is luminous_intensity / d².
    pub luminous_intensity: f64,
    // cd/m² of the star's disk.
    pub surface_luminance: f64,
}

pub fn star_light(star: &Star, radius: f64) -> StarLight {
    let spectrum = integrate_blackbody(star.effective_temperature);
    let radiance = STEFAN_BOLTZMANN * star.effective_temperature.powi(4) / PI;
    let luminous_efficacy = MAXIMUM_LUMINOUS_EFFICACY * spectrum.y / radiance;
    let luminous_flux = star.luminosity * luminous_efficacy;
    let surface_area = 4.0 * PI * radius * radius;
    StarLight {
        color: linear_srgb_from_xyz(spectrum / spectrum.y).max(DVec3::ZERO).as_vec3(),
        luminous_intensity: luminous_flux / (4.0 * PI),
        surface_luminance: luminous_flux / surface_area / PI,
    }
}

// CIE XYZ of a blackbody's spectral radiance, per steradian and square meter.
fn integrate_blackbody(temperature: f64) -> DVec3 {
    let steps = ((VISIBLE_END_NANOMETERS - VISIBLE_START_NANOMETERS) / INTEGRATION_STEP_NANOMETERS) as usize;
    (0..=steps)
        .map(|step| {
            let nanometers = VISIBLE_START_NANOMETERS + step as f64 * INTEGRATION_STEP_NANOMETERS;
            let meters = nanometers * 1e-9;
            let radiance = 2.0 * PLANCK * SPEED_OF_LIGHT * SPEED_OF_LIGHT
                / meters.powi(5)
                / ((PLANCK * SPEED_OF_LIGHT / (meters * BOLTZMANN * temperature)).exp() - 1.0);
            color_matching(nanometers) * radiance * INTEGRATION_STEP_NANOMETERS * 1e-9
        })
        .sum()
}

// Wyman, Sloan and Shirley's multi-lobe fit of the CIE 1931 2° color matching functions.
fn color_matching(nanometers: f64) -> DVec3 {
    let lobe = |mean: f64, below: f64, above: f64| {
        let width = if nanometers < mean { below } else { above };
        (-0.5 * ((nanometers - mean) / width).powi(2)).exp()
    };
    DVec3::new(
        1.056 * lobe(599.8, 37.9, 31.0) + 0.362 * lobe(442.0, 16.0, 26.7) - 0.065 * lobe(501.1, 20.4, 26.2),
        0.821 * lobe(568.8, 46.9, 40.5) + 0.286 * lobe(530.9, 16.3, 31.1),
        1.217 * lobe(437.0, 11.8, 36.0) + 0.681 * lobe(459.0, 26.0, 13.8),
    )
}

fn linear_srgb_from_xyz(xyz: DVec3) -> DVec3 {
    DVec3::new(
        3.2406 * xyz.x - 1.5372 * xyz.y - 0.4986 * xyz.z,
        -0.9689 * xyz.x + 1.8758 * xyz.y + 0.0415 * xyz.z,
        0.0557 * xyz.x - 0.2040 * xyz.y + 1.0570 * xyz.z,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;
    const SUN: Star = Star {
        luminosity: 3.828e26,
        effective_temperature: 5772.0,
    };
    const SUN_RADIUS: f64 = 6.957e8;

    #[test]
    fn sunlight_at_one_astronomical_unit_is_about_128_kilolux() {
        let light = star_light(&SUN, SUN_RADIUS);
        let illuminance = light.luminous_intensity / ASTRONOMICAL_UNIT.powi(2);
        assert!((illuminance - 128_000.0).abs() < 5_000.0, "{illuminance} lux");
    }

    #[test]
    fn sun_disk_luminance_is_about_two_billion_nits() {
        let luminance = star_light(&SUN, SUN_RADIUS).surface_luminance;
        assert!((1.6e9..2.2e9).contains(&luminance), "{luminance} cd/m²");
    }

    #[test]
    fn sunlight_is_a_slightly_warm_white() {
        let color = star_light(&SUN, SUN_RADIUS).color;
        assert!(color.x > color.z && color.z > 0.8 * color.x, "{color}");
    }
}
