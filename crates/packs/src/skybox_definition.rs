use std::path::PathBuf;

use anyhow::{Context, bail};
use glam::Mat3;
use xsa_core::frames;

use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack};

const SIMULATION_OBJECTS: &str = "objects/simulations";

pub struct SkyboxDefinition {
    pub texture: PathBuf,
    pub orientation: Mat3,
    pub luminance: f32,
}

impl SkyboxDefinition {
    pub fn load(stack: &PackStack, simulation: &Id) -> anyhow::Result<Option<SkyboxDefinition>> {
        let Ok(path) = stack.resource(SIMULATION_OBJECTS, simulation, DOCUMENT_EXTENSION) else {
            return Ok(None);
        };
        let object = Document::read(path)?;
        let Some(skybox) = object.optional_node("skybox") else {
            return Ok(None);
        };
        let parse = || -> anyhow::Result<SkyboxDefinition> {
            let texture = Id::parse(skybox.string_property("texture")?, &simulation.namespace)?;
            let orientation = match skybox.string_property("frame")? {
                "equatorial" => frames::equatorial_from_ecliptic().as_mat3(),
                "ecliptic" => Mat3::IDENTITY,
                other => bail!("unknown skybox frame `{other}`: use `equatorial` or `ecliptic`"),
            };

            Ok(SkyboxDefinition {
                texture: stack.texture(&texture)?.to_path_buf(),
                orientation,
                luminance: skybox.number_property("luminance")? as f32,
            })
        };

        parse()
            .map(Some)
            .with_context(|| format!("{}", object.path().display()))
    }
}
