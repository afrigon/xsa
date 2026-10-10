use std::path::PathBuf;

use anyhow::bail;
use kdl::KdlNode;

use super::{PolygonDefinition, RingDefinition};
use crate::{FromNode, NodeExtension, ParseContext};

pub enum GhostShape {
    Ring(RingDefinition),
    Polygon(PolygonDefinition),
    Texture(PathBuf),
}

impl FromNode for GhostShape {
    // Shapes are listed in preference order: the first whose resources a loaded pack provides is used.
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<GhostShape> {
        for shape in node.child_nodes() {
            match shape.node_name() {
                "ring" => return Ok(GhostShape::Ring(RingDefinition::from_node(shape, context)?)),
                "polygon" => return Ok(GhostShape::Polygon(PolygonDefinition::from_node(shape, context)?)),
                "texture" => {
                    let id = context.parse_id(shape.string_argument()?)?;

                    if let Ok(path) = context.stack.texture(&id) {
                        return Ok(GhostShape::Texture(path.to_path_buf()));
                    }

                    tracing::debug!("ghost texture {id} is not provided by any loaded pack: trying the next shape");
                }
                other => bail!("unknown ghost shape `{other}`: use `ring`, `polygon` or `texture`"),
            }
        }

        bail!("`ghost` needs a shape whose resources are available: `ring`, `polygon` or `texture`")
    }
}
