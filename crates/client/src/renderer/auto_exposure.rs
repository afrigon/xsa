use std::time::Instant;

const ISO_SENSITIVITY: f32 = 100.0;
// The incident-light meter calibration constant C, in lux·s.
const INCIDENT_METER_CALIBRATION: f32 = 250.0;
// Standard output sensitivity (ISO 12232): the exposure H = 10 / ISO lux·s renders middle gray. Through a lens
// transmitting q of the scene luminance L, H = q × L × 2^-EV, so middle gray is L = 10 / (ISO × q) × 2^EV.
const STANDARD_OUTPUT_EXPOSURE: f32 = 10.0;
const LENS_TRANSMITTANCE: f32 = 0.65;
// Display-referred middle gray, before tone mapping.
const MIDDLE_GRAY: f32 = 0.18;
// Softens jumps in starlight, such as switching the target from one planet to another.
pub(super) const EXPOSURE_TRANSITION_HALF_LIFE_SECONDS: f32 = 0.5;

pub struct AutoExposure {
    anchor_ev100: f32,
    ev100: f32,
    last_update: Option<Instant>,
    duration_scale: f32,
}

impl AutoExposure {
    pub fn new(initial_ev100: f32) -> Self {
        Self {
            anchor_ev100: initial_ev100,
            ev100: initial_ev100,
            last_update: None,
            duration_scale: 1.0,
        }
    }

    pub fn configure(&mut self, duration_scale: f32) {
        self.duration_scale = duration_scale;
    }

    pub fn ev100(&self) -> f32 {
        self.ev100
    }

    // The exposure an incident-light meter at the camera calls for.
    pub fn anchor_ev100(&self) -> f32 {
        self.anchor_ev100
    }

    // Starlight at the camera, in lux.
    pub fn update(&mut self, illuminance: f32) {
        let now = Instant::now();
        let elapsed = self.last_update.map(|last| now.duration_since(last).as_secs_f32());
        self.last_update = Some(now);
        self.anchor_ev100 = (illuminance * ISO_SENSITIVITY / INCIDENT_METER_CALIBRATION).log2();
        self.ev100 = match elapsed {
            Some(seconds) => self.transition(self.ev100, self.anchor_ev100, seconds),
            None => self.anchor_ev100,
        };
    }

    // Scales luminance (cd/m²) so that a scene metered at this EV100 lands mid-range, as a camera with ISO 100 would.
    pub fn exposure(&self) -> f32 {
        AutoExposure::exposure_for(self.ev100)
    }

    pub fn exposure_for(ev100: f32) -> f32 {
        let middle_gray_luminance = STANDARD_OUTPUT_EXPOSURE / (ISO_SENSITIVITY * LENS_TRANSMITTANCE) * ev100.exp2();
        MIDDLE_GRAY / middle_gray_luminance
    }

    fn transition(&self, current: f32, target: f32, seconds: f32) -> f32 {
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

    const SUNLIGHT_AT_ONE_ASTRONOMICAL_UNIT: f32 = 128_000.0;

    #[test]
    fn sunlight_at_one_astronomical_unit_meters_like_a_sunny_day() {
        let mut exposure = AutoExposure::new(0.0);
        exposure.update(SUNLIGHT_AT_ONE_ASTRONOMICAL_UNIT);
        assert!(
            (exposure.anchor_ev100() - 15.6).abs() < 0.1,
            "EV100 {}",
            exposure.anchor_ev100()
        );
    }

    #[test]
    fn a_gray_card_in_sunlight_renders_at_middle_gray() {
        const GRAY_CARD_ALBEDO: f32 = 0.18;
        let mut exposure = AutoExposure::new(0.0);
        exposure.update(SUNLIGHT_AT_ONE_ASTRONOMICAL_UNIT);
        let luminance = GRAY_CARD_ALBEDO * SUNLIGHT_AT_ONE_ASTRONOMICAL_UNIT / std::f32::consts::PI;
        let display = luminance * exposure.exposure();
        assert!((display - MIDDLE_GRAY).abs() < 0.02, "{display}");
    }

    #[test]
    fn a_transition_closes_half_the_gap_in_one_half_life() {
        let exposure = AutoExposure::new(0.0);
        let brighter = exposure.transition(0.0, 10.0, EXPOSURE_TRANSITION_HALF_LIFE_SECONDS);
        let darker = exposure.transition(10.0, 0.0, EXPOSURE_TRANSITION_HALF_LIFE_SECONDS);
        assert!((brighter - 5.0).abs() < 1e-4 && (darker - 5.0).abs() < 1e-4);
    }

    #[test]
    fn a_zero_duration_scale_transitions_at_once() {
        let mut exposure = AutoExposure::new(0.0);
        exposure.configure(0.0);
        assert_eq!(exposure.transition(0.0, 10.0, 0.0), 10.0);
    }
}
