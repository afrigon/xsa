const NEWTON_ITERATIONS: usize = 8;
const BISECTION_ITERATIONS: usize = 64;
const SOLVE_TOLERANCE: f64 = 1e-12;

/// A CSS-style cubic Bézier timing curve from (0, 0) to (1, 1) through the control points (x1, y1) and (x2, y2),
/// read as y for a given x.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl CubicBezier {
    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> CubicBezier {
        CubicBezier { x1, y1, x2, y2 }
    }

    /// The curve's y at `x` in 0..1.
    pub fn apply(&self, x: f64) -> f64 {
        CubicBezier::coordinate(self.y1, self.y2, self.parameter_at(x))
    }

    /// The slope dy/dx at `x` in 0..1.
    pub fn derivative(&self, x: f64) -> f64 {
        let parameter = self.parameter_at(x);

        CubicBezier::coordinate_derivative(self.y1, self.y2, parameter)
            / CubicBezier::coordinate_derivative(self.x1, self.x2, parameter)
    }

    // x(s) is monotonic for control points with x in 0..1, so Newton's method from s = x converges unless the slope
    // vanishes, where bisection takes over.
    fn parameter_at(&self, x: f64) -> f64 {
        let mut parameter = x;

        for _ in 0..NEWTON_ITERATIONS {
            let error = CubicBezier::coordinate(self.x1, self.x2, parameter) - x;

            if error.abs() < SOLVE_TOLERANCE {
                return parameter;
            }

            let slope = CubicBezier::coordinate_derivative(self.x1, self.x2, parameter);

            if slope.abs() < SOLVE_TOLERANCE {
                break;
            }

            parameter -= error / slope;
        }

        let (mut low, mut high) = (0.0, 1.0);
        parameter = x;

        for _ in 0..BISECTION_ITERATIONS {
            let error = CubicBezier::coordinate(self.x1, self.x2, parameter) - x;

            if error.abs() < SOLVE_TOLERANCE {
                break;
            }

            if error < 0.0 {
                low = parameter;
            } else {
                high = parameter;
            }

            parameter = (low + high) / 2.0;
        }

        parameter
    }

    fn coordinate(first: f64, second: f64, parameter: f64) -> f64 {
        let inverse = 1.0 - parameter;

        3.0 * inverse * inverse * parameter * first + 3.0 * inverse * parameter * parameter * second + parameter.powi(3)
    }

    fn coordinate_derivative(first: f64, second: f64, parameter: f64) -> f64 {
        let inverse = 1.0 - parameter;

        3.0 * inverse * inverse * first
            + 6.0 * inverse * parameter * (second - first)
            + 3.0 * parameter * parameter * (1.0 - second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f64 = 1e-9;

    #[test]
    fn linear_control_points_give_the_identity() {
        let curve = CubicBezier::new(0.25, 0.25, 0.75, 0.75);

        for x in [0.0, 0.1, 0.5, 0.9, 1.0] {
            assert!((curve.apply(x) - x).abs() < TOLERANCE);
            assert!((curve.derivative(x) - 1.0).abs() < TOLERANCE);
        }
    }

    #[test]
    fn a_symmetric_curve_crosses_its_middle_at_its_steepest() {
        let curve = CubicBezier::new(0.85, 0.0, 0.15, 1.0);

        assert!((curve.apply(0.5) - 0.5).abs() < TOLERANCE);
        assert!(curve.apply(0.0).abs() < TOLERANCE);
        assert!((curve.apply(1.0) - 1.0).abs() < TOLERANCE);
        assert!(curve.derivative(0.5) > curve.derivative(0.4));
        assert!(curve.derivative(0.0).abs() < TOLERANCE);
    }

    #[test]
    fn the_derivative_matches_a_finite_difference() {
        let curve = CubicBezier::new(0.85, 0.0, 0.15, 1.0);
        let step = 1e-6;

        for x in [0.2, 0.45, 0.7] {
            let difference = (curve.apply(x + step) - curve.apply(x - step)) / (2.0 * step);
            assert!((curve.derivative(x) - difference).abs() < 1e-4);
        }
    }
}
