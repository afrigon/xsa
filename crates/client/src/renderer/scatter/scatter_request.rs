use std::sync::Arc;

use xsa_core::scatter::ScatterPlacement;
use xsa_core::terrain::QuadSphereTile;

// A page the placement worker generates for a body scatter, by its index in the scene.
pub(in crate::renderer) struct ScatterRequest {
    pub scatter: usize,
    pub page: QuadSphereTile,
    pub placement: Arc<ScatterPlacement>,
}
