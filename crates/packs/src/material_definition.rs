use std::path::PathBuf;

use anyhow::{Context, bail, ensure};
use glam::Vec3;

use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack, ParseContext};

const MATERIALS: &str = "materials";
const BODY_OBJECTS: &str = "objects/bodies";

pub enum MaterialDefinition {
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
    // Hapke 2012's parameters other than the albedo, which comes from the color map: the particulate medium texture
    // (b, c, θ̄ in degrees, porosity K) and the opposition texture (B_S0, h_S, B_C0, h_C).
    Hapke {
        base_color: Vec3,
        color: Option<PathBuf>,
        normal: Option<PathBuf>,
        emissive: Option<PathBuf>,
        emissive_luminance: f32,
        medium: PathBuf,
        opposition: PathBuf,
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

        let shader_name = shader.to_string();

        match shader_name.as_str() {
            "base:emissive" => Ok(MaterialDefinition::Emissive {
                color: match document.optional_node("color") {
                    Some(_) => color("color")?,
                    None => Vec3::ONE,
                },
                luminance: number("luminance")?,
            }),
            "base:lambert" | "base:hapke" => {
                let base_color = color("base-color")?;
                let color = texture("color-texture")?;
                let normal = texture("normal-texture")?;
                let emissive = texture("emissive-texture")?;
                let emissive_luminance = number("emissive-luminance")?;
                ensure!(
                    emissive.is_none() || emissive_luminance.is_some(),
                    "an `emissive-texture` needs an `emissive-luminance` in cd/m²"
                );
                let emissive_luminance = emissive_luminance.unwrap_or(0.0);
                let required_texture = |name: &str| context.available_texture(document.node(name)?.string_argument()?);
                let (medium, opposition) = match shader_name.as_str() {
                    "base:hapke" => (
                        required_texture("medium-texture")?,
                        required_texture("opposition-texture")?,
                    ),
                    _ => (None, None),
                };

                // A Hapke material missing one of its maps is shaded with Lambert.
                match (medium, opposition) {
                    (Some(medium), Some(opposition)) => Ok(MaterialDefinition::Hapke {
                        base_color,
                        color,
                        normal,
                        emissive,
                        emissive_luminance,
                        medium,
                        opposition,
                    }),
                    _ => Ok(MaterialDefinition::Lambert {
                        base_color,
                        color,
                        normal,
                        emissive,
                        emissive_luminance,
                    }),
                }
            }
            _ => bail!("shader {shader} is not supported by materials yet"),
        }
    }
}
