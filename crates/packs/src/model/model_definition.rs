use std::path::PathBuf;

use anyhow::{Context, bail, ensure};

use super::ModelMaterial;
use crate::{DOCUMENT_EXTENSION, Document, FromNode, Id, NodeExtension, PackStack, ParseContext};

const MODEL_OBJECTS: &str = "objects/models";

// How a model looks: its mesh, and the material each of the mesh's nodes draws with, by node name.
pub struct ModelDefinition {
    pub mesh: PathBuf,
    pub materials: Vec<ModelMaterial>,
}

impl ModelDefinition {
    pub fn load(stack: &PackStack, model: &Id) -> anyhow::Result<ModelDefinition> {
        let document = Document::read(stack.resource(MODEL_OBJECTS, model, DOCUMENT_EXTENSION)?)?;
        let context = ParseContext { stack, id: model };

        ModelDefinition::parse(&document, &context).with_context(|| format!("{}", document.path().display()))
    }

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<ModelDefinition> {
        let mut materials: Vec<ModelMaterial> = Vec::new();

        for node in document.nodes() {
            match node.node_name() {
                "mesh" => {}
                "material" => {
                    let material = ModelMaterial::from_node(node, context)?;
                    ensure!(
                        materials.iter().all(|other| other.node != material.node),
                        "node `{}` has several materials",
                        material.node
                    );
                    materials.push(material);
                }
                other => bail!("unknown model node `{other}`: use `mesh` or `material`"),
            }
        }

        ensure!(
            !materials.is_empty(),
            "a model needs a `material` for each of its mesh's nodes"
        );

        let mesh = context.parse_id(document.node("mesh")?.string_argument()?)?;

        Ok(ModelDefinition {
            mesh: context.stack.mesh(&mesh)?.to_path_buf(),
            materials,
        })
    }
}
