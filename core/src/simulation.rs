mod barycenter_node;
mod body;
mod body_category;
mod body_description;
mod body_id;
mod body_index;
mod body_node;
mod body_state;
mod builder;
mod motion;
mod offset;
mod placement;
mod rotation;
mod simulation_state;
mod spin;
mod star;
mod system_node;
mod system_tree;

pub use barycenter_node::BarycenterNode;
pub use body::Body;
pub use body_category::BodyCategory;
pub use body_description::BodyDescription;
pub use body_id::BodyId;
pub use body_index::BodyIndex;
pub use body_node::BodyNode;
pub use body_state::BodyState;
pub use simulation_state::SimulationState;
pub use spin::Spin;
pub use star::Star;
pub use system_node::SystemNode;
pub use system_tree::SystemTree;

use anyhow::Context;
use glam::DVec3;
use xsa_units::SimulationTime;

use builder::Builder;
use offset::Offset;
use placement::Placement;
use rotation::Rotation;

pub struct Simulation {
    epoch: SimulationTime,
    bodies: Vec<Body>,
    spawn: Option<BodyIndex>,
    placements: Vec<Placement>,
    body_placements: Vec<usize>,
    rotations: Vec<Rotation>,
}

impl Simulation {
    pub fn new(system: &SystemTree, spawn: Option<&BodyId>) -> anyhow::Result<Simulation> {
        let mut builder = Builder {
            simulation: Simulation {
                epoch: system.epoch,
                bodies: Vec::new(),
                spawn: None,
                placements: Vec::new(),
                body_placements: Vec::new(),
                rotations: Vec::new(),
            },
        };
        builder.add_node(&system.root, None, None, 0.0)?;

        let mut simulation = builder.simulation;

        if let Some(spawn) = spawn {
            let index = simulation.find_body(spawn);
            simulation.spawn = Some(index.with_context(|| format!("the spawn body {spawn} is not in the system"))?);
        }

        Ok(simulation)
    }

    pub fn spawn(&self) -> Option<BodyIndex> {
        self.spawn
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn body(&self, index: BodyIndex) -> &Body {
        &self.bodies[index.value]
    }

    pub fn find_body(&self, id: &BodyId) -> Option<BodyIndex> {
        self.bodies
            .iter()
            .position(|body| body.id == *id)
            .map(|value| BodyIndex { value })
    }

    pub fn state_at(&self, time: SimulationTime, state: &mut SimulationState) {
        let seconds_since_epoch = time.seconds_since(self.epoch);
        let positions = &mut state.positions;
        positions.clear();

        for placement in &self.placements {
            let parent = placement.parent.map_or(DVec3::ZERO, |parent| positions[parent]);
            let offset = match &placement.offset {
                Offset::Origin => DVec3::ZERO,
                Offset::Orbit(motion) => motion.position(seconds_since_epoch),
                Offset::Share { motion, factor } => motion.position(seconds_since_epoch) * *factor,
            };
            positions.push(parent + offset);
        }

        state.bodies.clear();
        state.bodies.extend(
            self.body_placements
                .iter()
                .zip(&self.rotations)
                .map(|(&placement, rotation)| BodyState {
                    position: positions[placement],
                    orientation: rotation.orientation(seconds_since_epoch),
                }),
        );
    }
}
