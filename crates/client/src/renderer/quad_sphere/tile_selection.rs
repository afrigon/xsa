use glam::DVec3;

use super::{DetailRanges, HeightRange, HorizonCuller};
use crate::renderer::culler::Culler;

// What stays the same while one sphere's tiles are selected: the camera in the body's frame, every height the body
// has, and what decides a tile's detail and visibility.
pub(in crate::renderer) struct TileSelection<'a> {
    pub camera: DVec3,
    pub overall: HeightRange,
    pub ranges: &'a DetailRanges,
    pub culler: &'a Culler,
    pub horizon: HorizonCuller,
}
