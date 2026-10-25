use glam::DVec3;

#[derive(Clone, Copy)]
pub struct ViewPoint {
    pub focus: DVec3,
    pub distance: f64,
}

impl ViewPoint {
    pub fn between(focus: DVec3, position: DVec3) -> ViewPoint {
        ViewPoint {
            focus,
            distance: position.distance(focus),
        }
    }
}
