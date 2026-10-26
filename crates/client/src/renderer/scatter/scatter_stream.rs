use std::collections::{HashMap, HashSet};

use xsa_core::terrain::QuadSphereTile;

use super::scatter_page::ScatterPage;

// The pages of one body scatter within range of the camera, and those the worker is placing.
#[derive(Default)]
pub(in crate::renderer) struct ScatterStream {
    pub pages: HashMap<QuadSphereTile, ScatterPage>,
    pub requested: HashSet<QuadSphereTile>,
}
