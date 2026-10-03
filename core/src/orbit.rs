mod element_rates;

pub use element_rates::ElementRates;

use std::f64::consts::{PI, TAU};

use glam::{DQuat, DVec3};

const KEPLER_TOLERANCE: f64 = 1e-15;
const KEPLER_MAXIMUM_ITERATIONS: usize = 32;
const HIGH_ECCENTRICITY: f64 = 0.8;

#[derive(Clone, Copy, Debug)]
pub struct OrbitalElements {
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub ascending_node: f64,
    pub periapsis: f64,
    pub mean_anomaly: f64,
    pub mean_motion: Option<f64>,
    pub rates: ElementRates,
    pub reference_plane: DQuat,
}

impl OrbitalElements {
    pub fn mean_motion(&self, gravitational_parameter: f64) -> f64 {
        self.mean_motion
            .unwrap_or_else(|| (gravitational_parameter / self.semi_major_axis.powi(3)).sqrt())
    }

    pub fn plane_orientation(&self, seconds_since_epoch: f64) -> DQuat {
        let ascending_node = self.ascending_node + self.rates.ascending_node * seconds_since_epoch;
        let inclination = self.inclination + self.rates.inclination * seconds_since_epoch;
        self.reference_plane * DQuat::from_rotation_z(ascending_node) * DQuat::from_rotation_x(inclination)
    }

    pub fn periapsis_at(&self, seconds_since_epoch: f64) -> f64 {
        self.periapsis + self.rates.periapsis * seconds_since_epoch
    }

    pub fn mean_anomaly_at(&self, gravitational_parameter: f64, seconds_since_epoch: f64) -> f64 {
        self.mean_anomaly + self.mean_motion(gravitational_parameter) * seconds_since_epoch
    }

    pub fn position(&self, gravitational_parameter: f64, seconds_since_epoch: f64) -> DVec3 {
        let semi_major_axis = self.semi_major_axis + self.rates.semi_major_axis * seconds_since_epoch;
        let eccentricity = self.eccentricity + self.rates.eccentricity * seconds_since_epoch;
        let mean_anomaly = self.mean_anomaly_at(gravitational_parameter, seconds_since_epoch);
        let eccentric_anomaly = eccentric_anomaly(mean_anomaly, eccentricity);
        let in_plane = DVec3::new(
            semi_major_axis * (eccentric_anomaly.cos() - eccentricity),
            semi_major_axis * (1.0 - eccentricity * eccentricity).sqrt() * eccentric_anomaly.sin(),
            0.0,
        );
        let periapsis = DQuat::from_rotation_z(self.periapsis_at(seconds_since_epoch));
        self.plane_orientation(seconds_since_epoch) * periapsis * in_plane
    }
}

// Solves Kepler's equation M = E - e sin E for the eccentric anomaly E with Newton's method.
fn eccentric_anomaly(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let mean_anomaly = mean_anomaly.rem_euclid(TAU);
    let mut eccentric_anomaly = if eccentricity < HIGH_ECCENTRICITY {
        mean_anomaly
    } else {
        PI
    };

    for _ in 0..KEPLER_MAXIMUM_ITERATIONS {
        let correction = (eccentric_anomaly - eccentricity * eccentric_anomaly.sin() - mean_anomaly)
            / (1.0 - eccentricity * eccentric_anomaly.cos());
        eccentric_anomaly -= correction;

        if correction.abs() < KEPLER_TOLERANCE {
            break;
        }
    }

    eccentric_anomaly
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUN_GRAVITATIONAL_PARAMETER: f64 = 1.327_124_400_18e20;
    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;

    fn circular(semi_major_axis: f64) -> OrbitalElements {
        OrbitalElements {
            semi_major_axis,
            eccentricity: 0.0,
            inclination: 0.0,
            ascending_node: 0.0,
            periapsis: 0.0,
            mean_anomaly: 0.0,
            mean_motion: None,
            rates: ElementRates::default(),
            reference_plane: DQuat::IDENTITY,
        }
    }

    #[test]
    fn kepler_solution_satisfies_the_equation() {
        for eccentricity in [0.0, 0.1, 0.5, 0.9, 0.99] {
            for step in 0..64 {
                let mean_anomaly = step as f64 * TAU / 64.0;
                let solution = eccentric_anomaly(mean_anomaly, eccentricity);
                let residual = solution - eccentricity * solution.sin() - mean_anomaly;
                assert!(
                    residual.abs() < 1e-12,
                    "e={eccentricity} M={mean_anomaly}: residual {residual}"
                );
            }
        }
    }

    #[test]
    fn one_astronomical_unit_takes_about_a_year() {
        let orbit = circular(ASTRONOMICAL_UNIT);
        let period_days = TAU / orbit.mean_motion(SUN_GRAVITATIONAL_PARAMETER) / 86_400.0;
        assert!((period_days - 365.25).abs() < 0.1, "period {period_days} days");
    }

    #[test]
    fn periapsis_and_apoapsis_distances() {
        let mut orbit = circular(1.0e7);
        orbit.eccentricity = 0.5;
        let at_periapsis = orbit.position(1.0, 0.0).length();
        let half_period = PI / orbit.mean_motion(1.0);
        let at_apoapsis = orbit.position(1.0, half_period).length();
        assert!((at_periapsis - 0.5e7).abs() < 1e-3);
        assert!((at_apoapsis - 1.5e7).abs() < 1e-3);
    }

    #[test]
    fn inclined_orbit_rises_above_the_ecliptic() {
        let mut orbit = circular(1.0e7);
        orbit.inclination = 90_f64.to_radians();
        let quarter_period = PI / 2.0 / orbit.mean_motion(1.0);
        let position = orbit.position(1.0, quarter_period);
        assert!((position.z - 1.0e7).abs() < 1e-3, "{position}");
    }
}
