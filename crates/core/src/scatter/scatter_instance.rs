use glam::DVec2;

// One placed model: its offset from its page's center in the face's coordinates, its turn about its up axis
// (radians), its uniform scale, the share of its height below the ground, its lean from upright and the direction it
// leans toward (radians, from the turn's zero), and its model's place in the scatter's list.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ScatterInstance {
    pub offset: DVec2,
    pub yaw: f64,
    pub scale: f64,
    pub sink: f64,
    pub tilt: f64,
    pub tilt_heading: f64,
    pub model: usize,
}
