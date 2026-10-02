use std::f64::consts::{PI, TAU};

use anyhow::{Context, ensure};
use glam::{DQuat, DVec3};

use crate::orbit::OrbitalElements;
use crate::packs::data::{
    BarycenterNode, BodyDefinition, BodyNode, SimulationDefinition, Spin, SystemDefinition, SystemNode,
};
use crate::packs::{Id, PackStack};

pub struct Body {
    pub id: Id,
    pub radius: f64,
    pub gravitational_parameter: f64,
}

#[derive(Clone, Copy)]
pub struct BodyState {
    pub position: DVec3,
    pub orientation: DQuat,
}

#[derive(Default)]
pub struct SimulationState {
    positions: Vec<DVec3>,
    pub bodies: Vec<BodyState>,
}

pub struct Simulation {
    id: Id,
    epoch: f64,
    bodies: Vec<Body>,
    placements: Vec<Placement>,
    body_placements: Vec<usize>,
    rotations: Vec<Rotation>,
}

struct Placement {
    parent: Option<usize>,
    offset: Offset,
}

enum Offset {
    Origin,
    Orbit(Motion),
    Share { motion: Motion, factor: f64 },
}

#[derive(Clone, Copy)]
struct Motion {
    elements: OrbitalElements,
    gravitational_parameter: f64,
}

struct Rotation {
    reference: Option<Motion>,
    spin: Spin,
    axial_tilt: f64,
    period: f64,
}

impl Simulation {
    pub fn load(stack: &PackStack, id: &Id) -> anyhow::Result<Simulation> {
        let definition = SimulationDefinition::load(stack, id)?;
        let system = SystemDefinition::load(stack, &definition.system)?;
        let mut builder = Builder {
            stack,
            simulation: Simulation {
                id: id.clone(),
                epoch: system.epoch,
                bodies: Vec::new(),
                placements: Vec::new(),
                body_placements: Vec::new(),
                rotations: Vec::new(),
            },
        };
        builder.add_node(&system.root, None, 0.0)?;
        Ok(builder.simulation)
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn state_at(&self, time: f64, state: &mut SimulationState) {
        let seconds_since_epoch = time - self.epoch;
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

impl Motion {
    fn position(&self, seconds_since_epoch: f64) -> DVec3 {
        self.elements
            .position(self.gravitational_parameter, seconds_since_epoch)
    }
}

impl Rotation {
    fn orientation(&self, seconds_since_epoch: f64) -> DQuat {
        let plane = self.reference.map_or(DQuat::IDENTITY, |reference| {
            reference.elements.plane_orientation(seconds_since_epoch)
        });
        let angle = match self.reference {
            Some(reference) if self.spin.tidally_locked => {
                reference.elements.periapsis_at(seconds_since_epoch)
                    + reference
                        .elements
                        .mean_anomaly_at(reference.gravitational_parameter, seconds_since_epoch)
                    + PI
                    + self.spin.prime_meridian
            }
            _ => self.spin.prime_meridian + TAU * seconds_since_epoch / self.period,
        };
        plane
            * DQuat::from_rotation_z(self.spin.azimuth)
            * DQuat::from_rotation_x(self.axial_tilt)
            * DQuat::from_rotation_z(angle)
    }
}

struct Builder<'a> {
    stack: &'a PackStack,
    simulation: Simulation,
}

impl Builder<'_> {
    fn add_node(
        &mut self,
        node: &SystemNode,
        parent: Option<usize>,
        parent_gravitational_parameter: f64,
    ) -> anyhow::Result<()> {
        match node {
            SystemNode::Body(body) => self.add_orbiting_body(body, parent, parent_gravitational_parameter),
            SystemNode::Barycenter(barycenter) => {
                self.add_barycenter(barycenter, parent, parent_gravitational_parameter)
            }
        }
    }

    fn add_orbiting_body(
        &mut self,
        node: &BodyNode,
        parent: Option<usize>,
        parent_gravitational_parameter: f64,
    ) -> anyhow::Result<()> {
        let definition = BodyDefinition::load(self.stack, &node.body)?;
        let offset = match parent {
            None => {
                ensure!(
                    node.orbit.is_none(),
                    "{} is the system root and cannot orbit anything",
                    node.body
                );
                Offset::Origin
            }
            Some(_) => Offset::Orbit(Motion {
                elements: node.orbit.with_context(|| format!("{} needs an orbit", node.body))?,
                gravitational_parameter: parent_gravitational_parameter + definition.gravitational_parameter,
            }),
        };
        let reference = match &offset {
            Offset::Orbit(motion) => Some(*motion),
            _ => None,
        };
        self.add_body(node, definition, parent, offset, reference)
    }

    fn add_barycenter(
        &mut self,
        node: &BarycenterNode,
        parent: Option<usize>,
        parent_gravitational_parameter: f64,
    ) -> anyhow::Result<()> {
        let primary = BodyDefinition::load(self.stack, &node.primary.body)?;
        let secondary = BodyDefinition::load(self.stack, &node.secondary.body)?;
        let pair_gravitational_parameter = primary.gravitational_parameter + secondary.gravitational_parameter;
        let offset = match parent {
            None => {
                ensure!(
                    node.orbit.is_none(),
                    "barycenter {} is the system root and cannot orbit anything",
                    node.name
                );
                Offset::Origin
            }
            Some(_) => Offset::Orbit(Motion {
                elements: node
                    .orbit
                    .with_context(|| format!("barycenter {} needs an orbit", node.name))?,
                gravitational_parameter: parent_gravitational_parameter + pair_gravitational_parameter,
            }),
        };
        let barycenter_reference = match &offset {
            Offset::Orbit(motion) => Some(*motion),
            _ => None,
        };
        let barycenter = self.simulation.placements.len();
        self.simulation.placements.push(Placement { parent, offset });

        let relative_orbit = node
            .secondary
            .orbit
            .with_context(|| format!("the secondary of barycenter {} needs an orbit", node.name))?;
        let relative = Motion {
            elements: relative_orbit,
            gravitational_parameter: pair_gravitational_parameter,
        };
        let primary_factor = -secondary.gravitational_parameter / pair_gravitational_parameter;
        let secondary_factor = primary.gravitational_parameter / pair_gravitational_parameter;
        self.add_body(
            &node.primary,
            primary,
            Some(barycenter),
            Offset::Share {
                motion: relative,
                factor: primary_factor,
            },
            barycenter_reference,
        )?;
        self.add_body(
            &node.secondary,
            secondary,
            Some(barycenter),
            Offset::Share {
                motion: relative,
                factor: secondary_factor,
            },
            Some(relative),
        )?;
        for child in &node.children {
            self.add_node(child, Some(barycenter), pair_gravitational_parameter)?;
        }
        Ok(())
    }

    fn add_body(
        &mut self,
        node: &BodyNode,
        definition: BodyDefinition,
        parent: Option<usize>,
        offset: Offset,
        reference: Option<Motion>,
    ) -> anyhow::Result<()> {
        ensure!(
            !node.spin.tidally_locked || reference.is_some(),
            "{} cannot be tidally locked without an orbit",
            node.body
        );
        let placement = self.simulation.placements.len();
        self.simulation.placements.push(Placement { parent, offset });
        self.simulation.body_placements.push(placement);
        self.simulation.rotations.push(Rotation {
            reference,
            spin: node.spin,
            axial_tilt: definition.axial_tilt,
            period: definition.rotation_period,
        });
        let gravitational_parameter = definition.gravitational_parameter;
        self.simulation.bodies.push(Body {
            id: definition.id,
            radius: definition.radius,
            gravitational_parameter,
        });
        for child in &node.children {
            self.add_node(child, Some(placement), gravitational_parameter)?;
        }
        Ok(())
    }
}
