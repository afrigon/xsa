use std::f64::consts::TAU;

use glam::{DMat2, DVec2, Vec2};
use xsa_core::random::Seed;

// One rung of a surface's scale ladder, turned and shifted by its own seed so no two scales line up.
#[derive(Clone, Copy, Debug)]
pub(in crate::renderer) struct SurfaceScale {
    pub rotation: f64,
    pub offset: DVec2,
}

impl SurfaceScale {
    pub fn new(seed: Seed) -> SurfaceScale {
        SurfaceScale {
            rotation: seed.derive("rotation").fraction() * TAU,
            offset: DVec2::new(seed.derive("offset-x").fraction(), seed.derive("offset-y").fraction()),
        }
    }

    // Its rotation as the shader applies it: cosine and sine.
    pub fn rotation_data(&self) -> Vec2 {
        Vec2::new(self.rotation.cos() as f32, self.rotation.sin() as f32)
    }

    // Where `point` (m, on the projection plane) falls within this scale's repeat of `repeat` meters, in repeats.
    pub fn anchor(&self, point: DVec2, repeat: f64) -> Vec2 {
        let place = DMat2::from_angle(self.rotation) * point / repeat + self.offset;
        (place - place.floor()).as_vec2()
    }
}
