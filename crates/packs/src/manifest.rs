use std::path::Path;

use anyhow::{Context, ensure};
use semver::{Version, VersionReq};

use crate::{Dependency, Document, NodeExtension};

pub const PACK_FORMAT: u64 = 1;
pub const MANIFEST_FILE: &str = "pack.kdl";

pub struct Manifest {
    pub id: String,
    pub version: Version,
    pub format: u64,
    pub dependencies: Vec<Dependency>,
}

impl Manifest {
    pub fn read(pack_directory: &Path, id: &str) -> anyhow::Result<Manifest> {
        let path = pack_directory.join(MANIFEST_FILE);
        let manifest = Document::read(&path)?;
        Manifest::parse(&manifest, id).with_context(|| format!("{}", path.display()))
    }

    fn parse(manifest: &Document, id: &str) -> anyhow::Result<Manifest> {
        let version = manifest.node("version")?.string_argument()?;
        let version = Version::parse(version).with_context(|| format!("{version:?} is not a semantic version"))?;
        let format = manifest.node("format")?.number_argument()? as u64;
        ensure!(
            format == PACK_FORMAT,
            "pack format {format} is not supported (expected {PACK_FORMAT})"
        );
        let dependencies = match manifest.optional_node("dependencies") {
            None => Vec::new(),
            Some(node) => node
                .child_nodes()
                .iter()
                .map(|dependency| {
                    let requirement = dependency.string_argument()?;
                    Ok(Dependency {
                        pack: dependency.node_name().to_string(),
                        requirement: VersionReq::parse(requirement)
                            .with_context(|| format!("{requirement:?} is not a version requirement"))?,
                    })
                })
                .collect::<anyhow::Result<_>>()?,
        };
        Ok(Manifest {
            id: id.to_string(),
            version,
            format,
            dependencies,
        })
    }
}
