use super::{RandomRange, ScatterOrientation, ScatterPhysics, ScatterWeight};

// How one model of a scatter is placed: how it stands, how large it is, how deep it sinks into the ground and how far
// it leans, and how often it is picked over the others.
#[derive(Clone, Copy, Debug)]
pub struct ScatterModel {
    pub physics: ScatterPhysics,
    pub orientation: ScatterOrientation,
    // Uniform.
    pub scale: RandomRange,
    // The share of the model's height below the ground.
    pub sink: RandomRange,
    // Radians from upright, toward a random direction.
    pub tilt: RandomRange,
    pub weight: ScatterWeight,
}
