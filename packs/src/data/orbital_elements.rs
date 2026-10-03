use anyhow::{bail, ensure};
use glam::DQuat;
use kdl::KdlNode;
use xsa_core::frames;
use xsa_core::orbit::{ElementRates, OrbitalElements};
use xsa_units::SECONDS_PER_JULIAN_CENTURY;

use crate::{FromNode, NodeExtension, ParseContext};

impl FromNode for OrbitalElements {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<OrbitalElements> {
        let per_second = |per_century: f64| per_century / SECONDS_PER_JULIAN_CENTURY;
        let mut rates = ElementRates::default();
        let mut reference_plane = DQuat::IDENTITY;

        for child in node.child_nodes() {
            match child.node_name() {
                "rates" => {
                    let rate = |key| child.optional_number_property(key).map(Option::unwrap_or_default);
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
                        child.number_property("right-ascension")?.to_radians(),
                        child.number_property("declination")?.to_radians(),
                    );
                }
                other => bail!("unknown node `{other}` in orbit"),
            }
        }

        let eccentricity = node.number_property("eccentricity")?;
        ensure!(
            (0.0..1.0).contains(&eccentricity),
            "eccentricity {eccentricity} must be in [0, 1): only closed orbits are supported"
        );

        Ok(OrbitalElements {
            semi_major_axis: node.number_property("semi-major-axis")?,
            eccentricity,
            inclination: node.number_property("inclination")?.to_radians(),
            ascending_node: node.number_property("ascending-node")?.to_radians(),
            periapsis: node.number_property("periapsis")?.to_radians(),
            mean_anomaly: node.number_property("mean-anomaly")?.to_radians(),
            mean_motion: node
                .optional_number_property("mean-motion")?
                .map(|degrees_per_century| per_second(degrees_per_century.to_radians())),
            rates,
            reference_plane,
        })
    }
}
