use super::WeightNoise;

// How often a model is picked against the scatter's others; with noise, it varies over the ground in patches.
#[derive(Clone, Copy, Debug)]
pub struct ScatterWeight {
    pub value: f64,
    pub noise: Option<WeightNoise>,
}
