use std::f64::consts::TAU;

use glam::DQuat;

use super::client_world::ClientWorld;
use crate::renderer::Renderer;
use crate::ui::ZoomTransition;

const SPEED_UP: f64 = 600.0;

// A display-only turn of the main menu's body about its own axis, at its real rate sped up. When the menu leaves, the
// turn keeps going forward and stops on a whole turn, so the game shows the simulation's orientation again.
pub(super) struct MenuSpin {
    angle: f64,
    phase: Phase,
}

enum Phase {
    Idle,
    Spinning,
    Settling(Settle),
}

impl MenuSpin {
    pub fn new() -> MenuSpin {
        MenuSpin {
            angle: 0.0,
            phase: Phase::Idle,
        }
    }

    pub fn update(&mut self, world: &ClientWorld, menu_weight: f64, delta_seconds: f64) {
        let Some(body) = world.initial_target() else {
            return;
        };
        let speed = world.angular_speed(body) * SPEED_UP;

        if menu_weight >= 1.0 {
            self.angle = (self.angle + speed * delta_seconds).rem_euclid(TAU);
            self.phase = Phase::Spinning;

            return;
        }

        match &mut self.phase {
            Phase::Idle => {}
            Phase::Spinning => self.phase = Phase::Settling(Settle::new(self.angle, speed)),
            Phase::Settling(settle) => {
                settle.elapsed += delta_seconds;

                if settle.is_finished() {
                    self.angle = 0.0;
                    self.phase = Phase::Idle;
                } else {
                    self.angle = settle.angle();
                }
            }
        }
    }

    pub fn apply(&self, world: &ClientWorld, renderer: &mut Renderer) {
        let Some(body) = world.initial_target() else {
            return;
        };
        let object = renderer.scene_mut().object_mut(world.body_object(body));
        object.orientation *= DQuat::from_rotation_z(self.angle);
    }
}

// A cubic Hermite curve from the spin's angle and speed to rest on a whole turn, over the zoom's duration.
struct Settle {
    start_angle: f64,
    start_speed: f64,
    end_angle: f64,
    elapsed: f64,
}

impl Settle {
    fn new(start_angle: f64, start_speed: f64) -> Settle {
        // The curve only moves forward if the remaining angle is at least a third of the distance the starting speed
        // alone would cover; closer than that, it settles on the turn after.
        let minimum_remaining = start_speed * Settle::duration() / 3.0;
        let end_angle = if TAU - start_angle >= minimum_remaining {
            TAU
        } else {
            2.0 * TAU
        };

        Settle {
            start_angle,
            start_speed,
            end_angle,
            elapsed: 0.0,
        }
    }

    fn duration() -> f64 {
        f64::from(ZoomTransition::DURATION_SECONDS)
    }

    fn is_finished(&self) -> bool {
        self.elapsed >= Settle::duration()
    }

    fn angle(&self) -> f64 {
        let progress = self.elapsed / Settle::duration();
        let progress_squared = progress * progress;
        let progress_cubed = progress_squared * progress;
        let start_tangent_weight = progress_cubed - 2.0 * progress_squared + progress;
        let end_weight = 3.0 * progress_squared - 2.0 * progress_cubed;

        self.start_angle
            + start_tangent_weight * self.start_speed * Settle::duration()
            + end_weight * (self.end_angle - self.start_angle)
    }
}
