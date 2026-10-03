use anyhow::{Context, ensure};

use super::motion::Motion;
use super::offset::Offset;
use super::placement::Placement;
use super::rotation::Rotation;
use super::{BarycenterNode, Body, BodyNode, Simulation, SystemNode};

pub(super) struct Builder {
    pub simulation: Simulation,
}

impl Builder {
    pub fn add_node(
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
        let offset = match parent {
            None => {
                ensure!(
                    node.orbit.is_none(),
                    "{} is the system root and cannot orbit anything",
                    node.body.id
                );

                Offset::Origin
            }
            Some(_) => Offset::Orbit(Motion {
                elements: node.orbit.with_context(|| format!("{} needs an orbit", node.body.id))?,
                gravitational_parameter: parent_gravitational_parameter + node.body.gravitational_parameter,
            }),
        };
        let reference = offset.motion();

        self.add_body(node, parent, offset, reference)
    }

    fn add_barycenter(
        &mut self,
        node: &BarycenterNode,
        parent: Option<usize>,
        parent_gravitational_parameter: f64,
    ) -> anyhow::Result<()> {
        let primary_gravitational_parameter = node.primary.body.gravitational_parameter;
        let secondary_gravitational_parameter = node.secondary.body.gravitational_parameter;
        let pair_gravitational_parameter = primary_gravitational_parameter + secondary_gravitational_parameter;
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
        let barycenter_reference = offset.motion();
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
        let primary_factor = -secondary_gravitational_parameter / pair_gravitational_parameter;
        let secondary_factor = primary_gravitational_parameter / pair_gravitational_parameter;
        self.add_body(
            &node.primary,
            Some(barycenter),
            Offset::Share {
                motion: relative,
                factor: primary_factor,
            },
            barycenter_reference,
        )?;
        self.add_body(
            &node.secondary,
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
        parent: Option<usize>,
        offset: Offset,
        reference: Option<Motion>,
    ) -> anyhow::Result<()> {
        let body = &node.body;
        ensure!(
            !node.spin.tidally_locked || reference.is_some(),
            "{} cannot be tidally locked without an orbit",
            body.id
        );

        let placement = self.simulation.placements.len();
        self.simulation.placements.push(Placement { parent, offset });
        self.simulation.body_placements.push(placement);
        self.simulation.rotations.push(Rotation {
            reference,
            spin: node.spin,
            axial_tilt: body.axial_tilt,
            period: body.rotation_period,
        });
        self.simulation.bodies.push(Body {
            id: body.id.clone(),
            radius: body.radius,
            gravitational_parameter: body.gravitational_parameter,
            star: body.star,
        });

        for child in &node.children {
            self.add_node(child, Some(placement), body.gravitational_parameter)?;
        }

        Ok(())
    }
}
