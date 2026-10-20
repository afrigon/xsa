use std::path::PathBuf;

use anyhow::Context;

use crate::{DOCUMENT_EXTENSION, Document, Id, NodeExtension, PackStack, ParseContext};

const SURFACES: &str = "surfaces";

// A close-up ground material: three texture arrays of interchangeable variants, the side of the square one tile covers
// (m), the heights its displacement's 0 and 1 stand for (m), and how far it may be scaled up.
pub struct SurfaceDefinition {
    pub color: PathBuf,
    pub normal: PathBuf,
    pub displacement: PathBuf,
    pub size: f32,
    pub displacement_low: f32,
    pub displacement_high: f32,
    pub largest_magnification: f32,
}

impl SurfaceDefinition {
    // None, with a warning, while a texture it names is missing: surfaces are generated, and the ground draws without
    // them until they are.
    pub fn load(stack: &PackStack, surface: &Id) -> anyhow::Result<Option<SurfaceDefinition>> {
        let document = Document::read(stack.resource(SURFACES, surface, DOCUMENT_EXTENSION)?)?;
        let context = ParseContext { stack, id: surface };

        SurfaceDefinition::parse(&document, &context).with_context(|| format!("{}", document.path().display()))
    }

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<Option<SurfaceDefinition>> {
        let texture = |name: &str| context.available_texture(document.node(name)?.string_argument()?);
        let (Some(color), Some(normal), Some(displacement)) = (
            texture("color-texture")?,
            texture("normal-texture")?,
            texture("displacement-texture")?,
        ) else {
            return Ok(None);
        };
        let displacement_node = document.node("displacement")?;

        Ok(Some(SurfaceDefinition {
            color,
            normal,
            displacement,
            size: document.node("size")?.number_argument()? as f32,
            displacement_low: displacement_node.number_property("low")? as f32,
            displacement_high: displacement_node.number_property("high")? as f32,
            largest_magnification: document.node("largest-magnification")?.number_argument()? as f32,
        }))
    }
}
