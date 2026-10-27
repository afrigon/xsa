use std::path::PathBuf;

use anyhow::{Context, ensure};
use glam::Vec3;

use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack, ParseContext};

const MATERIALS: &str = "materials";

// A material's file, read by whichever family's definition it is for.
pub(super) struct MaterialDocument<'a> {
    pub document: Document,
    pub context: ParseContext<'a>,
}

impl<'a> MaterialDocument<'a> {
    pub fn read(stack: &'a PackStack, material: &'a Id) -> anyhow::Result<MaterialDocument<'a>> {
        Ok(MaterialDocument {
            document: Document::read(stack.resource(MATERIALS, material, DOCUMENT_EXTENSION)?)?,
            context: ParseContext { stack, id: material },
        })
    }

    pub fn shader(&self) -> anyhow::Result<String> {
        Ok(self
            .context
            .parse_id(self.document.node("shader")?.string_argument()?)?
            .to_string())
    }

    pub fn color(&self, name: &str) -> anyhow::Result<Vec3> {
        let components = self.document.node(name)?.number_arguments()?;
        ensure!(components.len() == 3, "`{name}` needs three components: red green blue");

        Ok(Vec3::new(
            components[0] as f32,
            components[1] as f32,
            components[2] as f32,
        ))
    }

    pub fn number(&self, name: &str) -> anyhow::Result<f32> {
        Ok(self.document.node(name)?.number_argument()? as f32)
    }

    pub fn optional_number(&self, name: &str) -> anyhow::Result<Option<f32>> {
        self.document
            .optional_node(name)
            .map(|node| node.number_argument().map(|value| value as f32))
            .transpose()
    }

    pub fn number_property(&self, name: &str, key: &str) -> anyhow::Result<f32> {
        Ok(self.document.node(name)?.number_property(key)? as f32)
    }

    pub fn texture(&self, name: &str) -> anyhow::Result<Option<PathBuf>> {
        match self.document.optional_node(name) {
            Some(node) => self.context.available_texture(node.string_argument()?),
            None => Ok(None),
        }
    }

    pub fn with_path<T>(&self, parsed: anyhow::Result<T>) -> anyhow::Result<T> {
        parsed.with_context(|| format!("{}", self.document.path().display()))
    }
}
