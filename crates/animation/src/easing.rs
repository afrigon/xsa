use crate::CubicBezier;

/// How a value moves from its start to its end: maps progress in 0..1 to eased progress in 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    /// Starts and ends slowly: 3p² − 2p³.
    Smoothstep,
    CubicBezier(CubicBezier),
}

impl Easing {
    /// The eased progress at `progress`, which is read as 0 below 0 and as 1 above 1.
    pub fn apply(&self, progress: f64) -> f64 {
        let progress = progress.clamp(0.0, 1.0);

        match self {
            Easing::Linear => progress,
            Easing::Smoothstep => progress * progress * (3.0 - 2.0 * progress),
            Easing::CubicBezier(curve) => curve.apply(progress),
        }
    }

    /// The rate of change of the eased progress per unit of progress at `progress` in 0..1.
    pub fn derivative(&self, progress: f64) -> f64 {
        let progress = progress.clamp(0.0, 1.0);

        match self {
            Easing::Linear => 1.0,
            Easing::Smoothstep => 6.0 * progress * (1.0 - progress),
            Easing::CubicBezier(curve) => curve.derivative(progress),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_easing_starts_at_zero_and_ends_at_one() {
        let easings = [
            Easing::Linear,
            Easing::Smoothstep,
            Easing::CubicBezier(CubicBezier::new(0.85, 0.0, 0.15, 1.0)),
        ];

        for easing in easings {
            assert!(easing.apply(0.0).abs() < 1e-9);
            assert!((easing.apply(1.0) - 1.0).abs() < 1e-9);
            assert!((easing.apply(2.0) - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn smoothstep_is_steepest_in_the_middle() {
        assert_eq!(Easing::Smoothstep.derivative(0.5), 1.5);
        assert_eq!(Easing::Smoothstep.derivative(0.0), 0.0);
    }
}
