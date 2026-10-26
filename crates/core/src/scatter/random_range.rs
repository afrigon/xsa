// Values drawn evenly between a minimum and a maximum, both included.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RandomRange {
    pub minimum: f64,
    pub maximum: f64,
}

impl RandomRange {
    pub const ZERO: RandomRange = RandomRange {
        minimum: 0.0,
        maximum: 0.0,
    };

    // The value `fraction` of the way across, from a fraction in [0, 1).
    pub fn at(self, fraction: f64) -> f64 {
        self.minimum + (self.maximum - self.minimum) * fraction
    }
}
