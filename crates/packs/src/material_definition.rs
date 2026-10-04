use std::path::PathBuf;

use anyhow::{Context, bail, ensure};
use glam::Vec3;

use crate::{DOCUMENT_EXTENSION, Document, FromNode, HapkeDefinition, Id, NodeExtension, PackStack, ParseContext};

const MATERIALS: &str = "materials";
const BODY_OBJECTS: &str = "objects/bodies";

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
        hapke: Option<HapkeDefinition>,
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
            let Some(node) = document.optional_node(name) else {
                return Ok(None);
            };
            let id = context.parse_id(node.string_argument()?)?;

            Ok(Some(context.stack.texture(&id)?.to_path_buf()))
        };

        match shader.to_string().as_str() {
            "base:lit" => Ok(MaterialDefinition::Lit {
                base_color: color("base-color")?,
            }),
            "base:emissive" => Ok(MaterialDefinition::Emissive {
                color: match document.optional_node("color") {
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
                    hapke: document
                        .optional_node("hapke")
                        .map(|node| HapkeDefinition::from_node(node, context))
                        .transpose()?,
                })
            }
            _ => bail!("shader {shader} is not supported by materials yet"),
        }
    }
}
