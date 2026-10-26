use xsa_core::scatter::ScatterInstance;
use xsa_core::terrain::QuadSphereTile;

// A page the placement worker generated.
pub(in crate::renderer) struct ScatterResult {
    pub scatter: usize,
    pub page: QuadSphereTile,
    pub instances: Vec<ScatterInstance>,
}
