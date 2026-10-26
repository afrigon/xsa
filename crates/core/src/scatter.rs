mod random_range;
mod scatter_grid;
mod scatter_instance;
mod scatter_model;
mod scatter_orientation;
mod scatter_physics;
mod scatter_placement;
mod scatter_weight;
mod weight_noise;

pub use random_range::RandomRange;
pub use scatter_grid::{PAGE_LEVELS, ScatterGrid};
pub use scatter_instance::ScatterInstance;
pub use scatter_model::ScatterModel;
pub use scatter_orientation::ScatterOrientation;
pub use scatter_physics::ScatterPhysics;
pub use scatter_placement::ScatterPlacement;
pub use scatter_weight::ScatterWeight;
pub use weight_noise::WeightNoise;

// Props placed on a body's ground: about one per `spacing` meters squared at full density, drawn within `range`
// meters of the camera, each one of `models`.
#[derive(Clone, Debug)]
pub struct Scatter {
    pub spacing: f64,
    pub range: f64,
    pub models: Vec<ScatterModel>,
}
