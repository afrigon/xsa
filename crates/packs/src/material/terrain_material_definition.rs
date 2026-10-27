use std::path::PathBuf;

use anyhow::{Context, bail, ensure};
use glam::Vec3;

use super::material_document::MaterialDocument;
use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack};

const BODY_OBJECTS: &str = "objects/bodies";

pub enum TerrainMaterialDefinition {
    // `color` and `normal` are terrain layers.
    Lambert {
        base_color: Vec3,
        color: Option<PathBuf>,
        normal: Option<PathBuf>,
        emissive: Option<PathBuf>,
        emissive_luminance: f32,
    },
    Emissive {
        color: Vec3,
        luminance: Option<f32>,
    },
    // A body's terrain shaded with Hapke 2012, its maps terrain layers. The albedo comes from the color layer; the
    // scattering layer holds the phase function's b and c, the surge layer the opposition effects' B_S0, h_S, B_C0
    // and h_C. The macroscopic roughness θ̄ (degrees) and the porosity coefficient K hold over the whole body. Without
    // both layers, it is shaded with Lambert.
    Hapke {
        base_color: Vec3,
        color: Option<PathBuf>,
        normal: Option<PathBuf>,
        emissive: Option<PathBuf>,
        emissive_luminance: f32,
        scattering: Option<PathBuf>,
        surge: Option<PathBuf>,
        macroscopic_roughness_degrees: f32,
        porosity: f32,
    },
}

impl TerrainMaterialDefinition {
    pub fn load_for_body(stack: &PackStack, body: &Id) -> anyhow::Result<TerrainMaterialDefinition> {
        let object = Document::read(stack.resource(BODY_OBJECTS, body, DOCUMENT_EXTENSION)?)?;
        let material = Id::parse(object.node("material")?.string_argument()?, &body.namespace)
            .with_context(|| format!("{}", object.path().display()))?;
        let document = MaterialDocument::read(stack, &material)?;

        document.with_path(TerrainMaterialDefinition::parse(&document))
    }

    fn parse(document: &MaterialDocument) -> anyhow::Result<TerrainMaterialDefinition> {
        let shader = document.shader()?;
        let emission = || -> anyhow::Result<(Option<PathBuf>, f32)> {
            let emissive = document.texture("emissive")?;
            let emissive_luminance = document.optional_number("emissive-luminance")?;
            ensure!(
                emissive.is_none() || emissive_luminance.is_some(),
                "an `emissive` texture needs an `emissive-luminance` in cd/m²"
            );

            Ok((emissive, emissive_luminance.unwrap_or(0.0)))
        };

        match shader.as_str() {
            "base:terrain/emissive" => Ok(TerrainMaterialDefinition::Emissive {
                color: match document.document.optional_node("color") {
                    Some(_) => document.color("color")?,
                    None => Vec3::ONE,
                },
                luminance: document.optional_number("luminance")?,
            }),
            "base:terrain/lambert" => {
                let (emissive, emissive_luminance) = emission()?;

                Ok(TerrainMaterialDefinition::Lambert {
                    base_color: document.color("base-color")?,
                    color: document.texture("color")?,
                    normal: document.texture("normal")?,
                    emissive,
                    emissive_luminance,
                })
            }
            "base:terrain/hapke" => {
                let (emissive, emissive_luminance) = emission()?;
                let scattering = document.texture("scattering")?;
                let surge = document.texture("surge")?;

                if scattering.is_none() || surge.is_none() {
                    tracing::warn!(
                        "{}: without both its scattering and surge layers, the terrain is shaded with Lambert",
                        document.context.id
                    );
                }

                Ok(TerrainMaterialDefinition::Hapke {
                    base_color: document.color("base-color")?,
                    color: document.texture("color")?,
                    normal: document.texture("normal")?,
                    emissive,
                    emissive_luminance,
                    scattering,
                    surge,
                    macroscopic_roughness_degrees: document.number("macroscopic-roughness")?,
                    porosity: document.number("porosity")?,
                })
            }
            _ => bail!("shader {shader} does not shade terrain: use a base:terrain/ shader"),
        }
    }
}
