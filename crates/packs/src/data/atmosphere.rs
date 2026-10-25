use anyhow::{bail, ensure};
use kdl::KdlNode;
use xsa_core::simulation::Atmosphere;

use crate::{FromNode, NodeExtension, ParseContext};

impl FromNode for Atmosphere {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<Atmosphere> {
        let mut twilight_depression = None;

        for child in node.child_nodes() {
            match child.node_name() {
                "twilight-depression" => {
                    let degrees = child.number_argument()?;
                    ensure!(
                        degrees > 0.0,
                        "twilight-depression {degrees} must be above 0; leave it out for an atmosphere without twilight"
                    );
                    twilight_depression = Some(degrees.to_radians());
                }
                other => bail!("unknown node `{other}` in atmosphere"),
            }
        }

        Ok(Atmosphere { twilight_depression })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use kdl::KdlDocument;

    use super::*;
    use crate::{Id, PackStack};

    fn parse(text: &str) -> anyhow::Result<Atmosphere> {
        let document: KdlDocument = text.parse().unwrap();
        let stack = PackStack::load(Path::new("."), &[]).unwrap();
        let id = Id::parse("test:body", "test").unwrap();

        Atmosphere::from_node(&document.nodes()[0], &ParseContext { stack: &stack, id: &id })
    }

    #[test]
    fn the_twilight_depression_reads_in_radians() {
        let atmosphere = parse("atmosphere { twilight-depression 6 }").unwrap();
        assert_eq!(atmosphere.twilight_depression, Some(6.0_f64.to_radians()));
    }

    #[test]
    fn an_atmosphere_without_twilight_has_none() {
        assert_eq!(parse("atmosphere").unwrap().twilight_depression, None);
    }

    #[test]
    fn a_twilight_depression_must_be_above_zero() {
        assert!(parse("atmosphere { twilight-depression 0 }").is_err());
        assert!(parse("atmosphere { twilight-depression -6 }").is_err());
    }
}
