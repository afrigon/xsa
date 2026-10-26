use std::path::PathBuf;

use anyhow::{Context, bail, ensure};
use glam::Vec3;

use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack, ParseContext};

const MATERIALS: &str = "materials";
const BODY_OBJECTS: &str = "objects/bodies";

pub enum MaterialDefinition {
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

impl MaterialDefinition {
    pub fn load_for_body(stack: &PackStack, body: &Id) -> anyhow::Result<MaterialDefinition> {
        let object = Document::read(stack.resource(BODY_OBJECTS, body, DOCUMENT_EXTENSION)?)?;
        let material = Id::parse(object.node("material")?.string_argument()?, &body.namespace)
            .with_context(|| format!("{}", object.path().display()))?;
        let document = Document::read(stack.resource(MATERIALS, &material, DOCUMENT_EXTENSION)?)?;
        let context = ParseContext { stack, id: &material };

        MaterialDefinition::parse(&document, &context).with_context(|| format!("{}", document.path().display()))
    }

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<MaterialDefinition> {
        let shader = context.parse_id(document.node("shader")?.string_argument()?)?;
        let color = |name: &str| -> anyhow::Result<Vec3> {
            let components = document.node(name)?.number_arguments()?;
            ensure!(components.len() == 3, "`{name}` needs three components: red green blue");

            Ok(Vec3::new(
                components[0] as f32,
                components[1] as f32,
                components[2] as f32,
            ))
        };
        let number = |name: &str| -> anyhow::Result<Option<f32>> {
            document
                .optional_node(name)
                .map(|node| node.number_argument().map(|value| value as f32))
                .transpose()
        };
        let texture = |name: &str| -> anyhow::Result<Option<PathBuf>> {
            match document.optional_node(name) {
                Some(node) => context.available_texture(node.string_argument()?),
                None => Ok(None),
            }
        };
        let emission = || -> anyhow::Result<(Option<PathBuf>, f32)> {
            let emissive = texture("emissive")?;
            let emissive_luminance = number("emissive-luminance")?;
            ensure!(
                emissive.is_none() || emissive_luminance.is_some(),
                "an `emissive` texture needs an `emissive-luminance` in cd/m²"
            );

            Ok((emissive, emissive_luminance.unwrap_or(0.0)))
        };

        let shader_name = shader.to_string();

        match shader_name.as_str() {
            "base:terrain/emissive" => Ok(MaterialDefinition::Emissive {
                color: match document.optional_node("color") {
                    Some(_) => color("color")?,
                    None => Vec3::ONE,
                },
                luminance: number("luminance")?,
            }),
            "base:terrain/lambert" => {
                let (emissive, emissive_luminance) = emission()?;

                Ok(MaterialDefinition::Lambert {
                    base_color: color("base-color")?,
                    color: texture("color")?,
                    normal: texture("normal")?,
                    emissive,
                    emissive_luminance,
                })
            }
            "base:terrain/hapke" => {
                let (emissive, emissive_luminance) = emission()?;
                let scattering = texture("scattering")?;
                let surge = texture("surge")?;

                if scattering.is_none() || surge.is_none() {
                    tracing::warn!(
                        "{}: without both its scattering and surge layers, the terrain is shaded with Lambert",
                        context.id
                    );
                }

                Ok(MaterialDefinition::Hapke {
                    base_color: color("base-color")?,
                    color: texture("color")?,
                    normal: texture("normal")?,
                    emissive,
                    emissive_luminance,
                    scattering,
                    surge,
                    macroscopic_roughness_degrees: document.node("macroscopic-roughness")?.number_argument()? as f32,
                    porosity: document.node("porosity")?.number_argument()? as f32,
                })
            }
            _ => bail!("shader {shader} is not supported by materials yet"),
        }
    }
}
