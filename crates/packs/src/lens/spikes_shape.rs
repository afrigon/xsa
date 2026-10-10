use std::path::PathBuf;

use anyhow::bail;
use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

pub enum SpikesShape {
    Computed { width: f32, dispersion: f32 },
    Texture(PathBuf),
}

impl FromNode for SpikesShape {
    // Shapes are listed in preference order: the first whose resources a loaded pack provides is used.
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<SpikesShape> {
        for shape in node.child_nodes() {
            match shape.node_name() {
                "computed" => {
                    return Ok(SpikesShape::Computed {
                        width: shape.number_property("width")? as f32,
                        dispersion: shape.number_property("dispersion")? as f32,
                    });
                }
                "texture" => {
                    let id = context.parse_id(shape.string_argument()?)?;

                    if let Ok(path) = context.stack.texture(&id) {
                        return Ok(SpikesShape::Texture(path.to_path_buf()));
                    }

                    tracing::debug!("spikes texture {id} is not provided by any loaded pack: trying the next shape");
                }
                other => bail!("unknown spikes shape `{other}`: use `computed` or `texture`"),
            }
        }

        bail!("`spikes` needs a shape whose resources are available: `computed` or `texture`")
    }
}
