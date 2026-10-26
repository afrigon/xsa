use glam::DVec3;

use super::{DetailRanges, HorizonCuller};
use crate::renderer::culler::Culler;

// What stays the same while one sphere's tiles are selected: the camera in the body's frame, and what decides a tile's
// detail and visibility.
pub(in crate::renderer) struct TileSelection<'a> {
    pub camera: DVec3,
    pub ranges: &'a DetailRanges,
    pub culler: &'a Culler,
    pub horizon: HorizonCuller,
}
