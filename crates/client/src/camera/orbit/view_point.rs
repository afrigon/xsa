use glam::DVec3;

#[derive(Clone, Copy)]
pub(super) struct ViewPoint {
    pub focus: DVec3,
    pub distance: f64,
}
