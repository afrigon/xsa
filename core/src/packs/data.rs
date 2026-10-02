use anyhow::{Context, bail, ensure};
use glam::DQuat;
use kdl::KdlNode;

use super::document::{self, Document};
use super::id::Id;
use super::stack::PackStack;
use crate::frames;
use crate::orbit::{ElementRates, OrbitalElements};
use crate::time::{self, SECONDS_PER_JULIAN_CENTURY};

pub const BODIES: &str = "bodies";
pub const SYSTEMS: &str = "systems";
pub const SIMULATIONS: &str = "simulations";

pub struct BodyDefinition {
    pub id: Id,
    pub radius: f64,
    pub gravitational_parameter: f64,
    pub rotation_period: f64,
    pub axial_tilt: f64,
    pub star: Option<Star>,
}

#[derive(Clone, Copy, Debug)]
pub struct Star {
    pub luminosity: f64,
    pub effective_temperature: f64,
}

pub struct SystemDefinition {
    pub id: Id,
    pub epoch: f64,
    pub root: SystemNode,
}

pub enum SystemNode {
    Body(Box<BodyNode>),
    Barycenter(Box<BarycenterNode>),
}

pub struct BodyNode {
    pub body: Id,
    pub orbit: Option<OrbitalElements>,
    pub spin: Spin,
    pub children: Vec<SystemNode>,
}

pub struct BarycenterNode {
    pub name: String,
    pub orbit: Option<OrbitalElements>,
    pub primary: BodyNode,
    pub secondary: BodyNode,
    pub children: Vec<SystemNode>,
}

#[derive(Clone, Copy, Default)]
pub struct Spin {
    pub tidally_locked: bool,
    pub azimuth: f64,
    pub prime_meridian: f64,
}

pub struct SimulationDefinition {
    pub id: Id,
    pub system: Id,
    pub spawn: Option<Id>,
}

impl BodyDefinition {
    pub fn load(stack: &PackStack, id: &Id) -> anyhow::Result<BodyDefinition> {
        let file = Document::read(stack.data(BODIES, id)?)?;
        let parse = || -> anyhow::Result<BodyDefinition> {
            Ok(BodyDefinition {
                id: id.clone(),
                radius: document::number_argument(file.node("radius")?)?,
                gravitational_parameter: document::number_argument(file.node("gravitational-parameter")?)?,
                rotation_period: document::number_argument(file.node("rotation-period")?)?,
                axial_tilt: document::number_argument(file.node("axial-tilt")?)?.to_radians(),
                star: parse_star(&file)?,
            })
        };
        parse().with_context(|| format!("{}", file.path().display()))
    }
}

impl SystemDefinition {
    pub fn load(stack: &PackStack, id: &Id) -> anyhow::Result<SystemDefinition> {
        let file = Document::read(stack.data(SYSTEMS, id)?)?;
        let parse = || -> anyhow::Result<SystemDefinition> {
            let epoch = time::parse_timestamp(document::string_argument(file.node("epoch")?)?)?;
            let mut roots = file.nodes().iter().filter(|node| document::name(node) != "epoch");
            let root = roots.next().context("a system needs a `star` or `barycenter` root")?;
            ensure!(roots.next().is_none(), "a system has exactly one root");
            let root = match document::name(root) {
                "star" => SystemNode::Body(Box::new(parse_body(root, &id.namespace)?)),
                "barycenter" => SystemNode::Barycenter(Box::new(parse_barycenter(root, &id.namespace)?)),
                other => bail!("unknown system root `{other}`"),
            };
            Ok(SystemDefinition {
                id: id.clone(),
                epoch,
                root,
            })
        };
        parse().with_context(|| format!("{}", file.path().display()))
    }
}

impl SimulationDefinition {
    pub fn load(stack: &PackStack, id: &Id) -> anyhow::Result<SimulationDefinition> {
        let file = Document::read(stack.data(SIMULATIONS, id)?)?;
        let parse = || -> anyhow::Result<SimulationDefinition> {
            Ok(SimulationDefinition {
                id: id.clone(),
                system: Id::parse(document::string_argument(file.node("system")?)?, &id.namespace)?,
                spawn: file
                    .optional_node("spawn")
                    .map(|node| Id::parse(document::string_argument(node)?, &id.namespace))
                    .transpose()?,
            })
        };
        parse().with_context(|| format!("{}", file.path().display()))
    }
}

fn parse_star(file: &Document) -> anyhow::Result<Option<Star>> {
    let luminosity = file.optional_node("luminosity");
    let temperature = file.optional_node("effective-temperature");
    ensure!(
        luminosity.is_some() == temperature.is_some(),
        "a star needs both `luminosity` and `effective-temperature`"
    );
    let Some(luminosity) = luminosity else {
        return Ok(None);
    };
    let Some(temperature) = temperature else {
        return Ok(None);
    };
    Ok(Some(Star {
        luminosity: document::number_argument(luminosity)?,
        effective_temperature: document::number_argument(temperature)?,
    }))
}

fn parse_body(node: &KdlNode, namespace: &str) -> anyhow::Result<BodyNode> {
    let body = Id::parse(document::string_argument(node)?, namespace)?;
    let mut parsed = BodyNode {
        body,
        orbit: None,
        spin: Spin {
            tidally_locked: document::bool_property(node, "tidally-locked")?,
            azimuth: degrees_property(node, "spin-azimuth")?,
            prime_meridian: degrees_property(node, "prime-meridian")?,
        },
        children: Vec::new(),
    };
    for child in document::children(node) {
        match document::name(child) {
            "orbit" => parsed.orbit = Some(parse_orbit(child)?),
            "body" => parsed
                .children
                .push(SystemNode::Body(Box::new(parse_body(child, namespace)?))),
            "barycenter" => parsed
                .children
                .push(SystemNode::Barycenter(Box::new(parse_barycenter(child, namespace)?))),
            other => bail!("unknown node `{other}` in body {}", parsed.body),
        }
    }
    Ok(parsed)
}

fn parse_barycenter(node: &KdlNode, namespace: &str) -> anyhow::Result<BarycenterNode> {
    let name = document::string_argument(node)?.to_string();
    let mut orbit = None;
    let mut primary = None;
    let mut secondary = None;
    let mut children = Vec::new();
    for child in document::children(node) {
        match document::name(child) {
            "orbit" => orbit = Some(parse_orbit(child)?),
            "primary" => primary = Some(parse_body(child, namespace)?),
            "secondary" => secondary = Some(parse_body(child, namespace)?),
            "body" => children.push(SystemNode::Body(Box::new(parse_body(child, namespace)?))),
            "barycenter" => children.push(SystemNode::Barycenter(Box::new(parse_barycenter(child, namespace)?))),
            other => bail!("unknown node `{other}` in barycenter {name}"),
        }
    }
    let primary = primary.with_context(|| format!("barycenter {name} needs a `primary`"))?;
    let secondary = secondary.with_context(|| format!("barycenter {name} needs a `secondary`"))?;
    ensure!(
        primary.orbit.is_none(),
        "the primary of barycenter {name} has no orbit; the secondary's orbit is relative to it"
    );
    ensure!(
        secondary.orbit.is_some(),
        "the secondary of barycenter {name} needs an orbit relative to the primary"
    );
    Ok(BarycenterNode {
        name,
        orbit,
        primary,
        secondary,
        children,
    })
}

fn parse_orbit(node: &KdlNode) -> anyhow::Result<OrbitalElements> {
    let per_second = |per_century: f64| per_century / SECONDS_PER_JULIAN_CENTURY;
    let mut rates = ElementRates::default();
    let mut reference_plane = DQuat::IDENTITY;
    for child in document::children(node) {
        match document::name(child) {
            "rates" => {
                let rate = |key| document::optional_number_property(child, key).map(Option::unwrap_or_default);
                rates = ElementRates {
                    semi_major_axis: per_second(rate("semi-major-axis")?),
                    eccentricity: per_second(rate("eccentricity")?),
                    inclination: per_second(rate("inclination")?.to_radians()),
                    ascending_node: per_second(rate("ascending-node")?.to_radians()),
                    periapsis: per_second(rate("periapsis")?.to_radians()),
                };
            }
            "plane" => {
                reference_plane = frames::plane_from_equatorial_pole(
                    document::number_property(child, "right-ascension")?.to_radians(),
                    document::number_property(child, "declination")?.to_radians(),
                );
            }
            other => bail!("unknown node `{other}` in orbit"),
        }
    }
    let eccentricity = document::number_property(node, "eccentricity")?;
    ensure!(
        (0.0..1.0).contains(&eccentricity),
        "eccentricity {eccentricity} must be in [0, 1): only closed orbits are supported"
    );
    Ok(OrbitalElements {
        semi_major_axis: document::number_property(node, "semi-major-axis")?,
        eccentricity,
        inclination: document::number_property(node, "inclination")?.to_radians(),
        ascending_node: document::number_property(node, "ascending-node")?.to_radians(),
        periapsis: document::number_property(node, "periapsis")?.to_radians(),
        mean_anomaly: document::number_property(node, "mean-anomaly")?.to_radians(),
        mean_motion: document::optional_number_property(node, "mean-motion")?
            .map(|degrees_per_century| per_second(degrees_per_century.to_radians())),
        rates,
        reference_plane,
    })
}

fn degrees_property(node: &KdlNode, key: &str) -> anyhow::Result<f64> {
    Ok(document::optional_number_property(node, key)?
        .unwrap_or_default()
        .to_radians())
}
