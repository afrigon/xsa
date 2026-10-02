use std::fs;
use std::path::PathBuf;

use anyhow::{Context, bail, ensure};
use glam::{Mat3, Vec3};
use xsa_core::frames;
use xsa_core::packs::document::{self, Document};
use xsa_core::packs::{Id, PackStack};

use crate::renderer::{HISTOGRAM_SHADER_PATH, POINT_SHADER_PATH, Shader, ShaderBinaries, TONEMAP_SHADER_PATH};

const BASE_NAMESPACE: &str = "base";
const SHADERS: &str = "shaders";
const SHADER_EXTENSION: &str = "spv";
const MATERIALS: &str = "materials";
const BODY_OBJECTS: &str = "objects/bodies";
const SIMULATION_OBJECTS: &str = "objects/simulations";
const TEXTURES: &str = "textures";
const TEXTURE_EXTENSION: &str = "dds";
const DEFINITION_EXTENSION: &str = "kdl";

pub enum MaterialDefinition {
    Lit {
        base_color: Vec3,
    },
    Emissive {
        color: Vec3,
        luminance: Option<f32>,
    },
    Planet {
        color: PathBuf,
        normal: Option<PathBuf>,
        emissive: Option<PathBuf>,
        emissive_luminance: f32,
    },
}

pub struct SkyboxDefinition {
    pub texture: PathBuf,
    pub orientation: Mat3,
    pub luminance: f32,
}

pub fn load_shaders(stack: &PackStack) -> anyhow::Result<ShaderBinaries> {
    let read = |path: &str| {
        let id = Id {
            namespace: BASE_NAMESPACE.to_string(),
            path: path.to_string(),
        };
        let file = stack
            .resource(SHADERS, &id, SHADER_EXTENSION)
            .context("shaders are missing: run `mise run shaders`")?;
        fs::read(file).with_context(|| format!("reading {}", file.display()))
    };
    Ok(ShaderBinaries {
        shaders: Shader::ALL
            .iter()
            .map(|shader| read(shader.path()))
            .collect::<anyhow::Result<_>>()?,
        point: read(POINT_SHADER_PATH)?,
        tonemap: read(TONEMAP_SHADER_PATH)?,
        histogram: read(HISTOGRAM_SHADER_PATH)?,
    })
}

pub fn load_body_material(stack: &PackStack, body: &Id) -> anyhow::Result<MaterialDefinition> {
    let object = Document::read(stack.resource(BODY_OBJECTS, body, DEFINITION_EXTENSION)?)?;
    let material = Id::parse(document::string_argument(object.node("material")?)?, &body.namespace)
        .with_context(|| format!("{}", object.path().display()))?;
    let file = Document::read(stack.resource(MATERIALS, &material, DEFINITION_EXTENSION)?)?;
    parse_material(stack, &file, &material.namespace).with_context(|| format!("{}", file.path().display()))
}

fn parse_material(stack: &PackStack, file: &Document, namespace: &str) -> anyhow::Result<MaterialDefinition> {
    let shader = Id::parse(document::string_argument(file.node("shader")?)?, namespace)?;
    let color = |name: &str| -> anyhow::Result<Vec3> {
        let components = document::number_arguments(file.node(name)?)?;
        ensure!(components.len() == 3, "`{name}` needs three components: red green blue");
        Ok(Vec3::new(
            components[0] as f32,
            components[1] as f32,
            components[2] as f32,
        ))
    };
    let number = |name: &str| -> anyhow::Result<Option<f32>> {
        file.optional_node(name)
            .map(|node| document::number_argument(node).map(|value| value as f32))
            .transpose()
    };
    let texture = |name: &str| -> anyhow::Result<Option<PathBuf>> {
        let Some(node) = file.optional_node(name) else {
            return Ok(None);
        };
        let id = Id::parse(document::string_argument(node)?, namespace)?;
        Ok(Some(stack.resource(TEXTURES, &id, TEXTURE_EXTENSION)?.to_path_buf()))
    };
    match shader.to_string().as_str() {
        "base:lit" => Ok(MaterialDefinition::Lit {
            base_color: color("base-color")?,
        }),
        "base:emissive" => Ok(MaterialDefinition::Emissive {
            color: match file.optional_node("color") {
                Some(_) => color("color")?,
                None => Vec3::ONE,
            },
            luminance: number("luminance")?,
        }),
        "base:planet" => {
            let emissive = texture("emissive-texture")?;
            let emissive_luminance = number("emissive-luminance")?;
            ensure!(
                emissive.is_none() || emissive_luminance.is_some(),
                "an `emissive-texture` needs an `emissive-luminance` in cd/m²"
            );
            Ok(MaterialDefinition::Planet {
                color: texture("color-texture")?.context("a planet material needs a `color-texture`")?,
                normal: texture("normal-texture")?,
                emissive,
                emissive_luminance: emissive_luminance.unwrap_or(0.0),
            })
        }
        _ => bail!("shader {shader} is not supported by materials yet"),
    }
}

pub fn load_skybox(stack: &PackStack, simulation: &Id) -> anyhow::Result<Option<SkyboxDefinition>> {
    let Ok(path) = stack.resource(SIMULATION_OBJECTS, simulation, DEFINITION_EXTENSION) else {
        return Ok(None);
    };
    let object = Document::read(path)?;
    let Some(skybox) = object.optional_node("skybox") else {
        return Ok(None);
    };
    let parse = || -> anyhow::Result<SkyboxDefinition> {
        let texture = Id::parse(document::string_property(skybox, "texture")?, &simulation.namespace)?;
        let orientation = match document::string_property(skybox, "frame")? {
            "equatorial" => frames::equatorial_from_ecliptic().as_mat3(),
            "ecliptic" => Mat3::IDENTITY,
            other => bail!("unknown skybox frame `{other}`: use `equatorial` or `ecliptic`"),
        };
        Ok(SkyboxDefinition {
            texture: stack.resource(TEXTURES, &texture, TEXTURE_EXTENSION)?.to_path_buf(),
            orientation,
            luminance: document::number_property(skybox, "luminance")? as f32,
        })
    };
    parse()
        .map(Some)
        .with_context(|| format!("{}", object.path().display()))
}
