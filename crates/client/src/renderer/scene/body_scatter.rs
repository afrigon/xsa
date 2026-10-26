use std::sync::Arc;

use glam::Vec3;
use xsa_core::scatter::ScatterPlacement;

use super::{ModelHandle, TerrainHandle};

// A scatter placed on a body's terrain: its models in the order its placement picks them, the multiplier on their
// base colour, and the ground's average albedo (linear), which lights them until the terrain's colour layer loads.
pub struct BodyScatter {
    pub terrain: TerrainHandle,
    pub placement: Arc<ScatterPlacement>,
    pub models: Vec<ModelHandle>,
    pub tint: Vec3,
    pub ground_albedo: Vec3,
}
