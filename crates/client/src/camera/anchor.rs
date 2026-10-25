use glam::DVec3;
use xsa_core::simulation::BodyIndex;

use crate::app::ClientWorld;

// What the camera is attached to, so that it moves with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Body(BodyIndex),
}

impl Anchor {
    // The body the camera is about.
    pub fn body(self) -> BodyIndex {
        match self {
            Anchor::Body(body) => body,
        }
    }

    pub fn position(self, world: &ClientWorld) -> DVec3 {
        match self {
            Anchor::Body(body) => world.body_state(body).position,
        }
    }
}
