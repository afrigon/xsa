use glam::DVec3;

use crate::renderer::Scene;
use crate::renderer::material::MaterialBuffer;
use crate::renderer::temporal_history::TemporalHistory;

// What a frame's scatters are placed from.
pub(in crate::renderer) struct ScatterView<'a> {
    pub scene: &'a Scene,
    pub materials: &'a [MaterialBuffer],
    pub temporal: &'a TemporalHistory,
    pub camera: DVec3,
    pub height_scale: f64,
    // Off, every scatter drops its pages.
    pub enabled: bool,
}
