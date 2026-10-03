use anyhow::{Context, bail};
use kdl::{KdlNode, KdlValue};

use crate::{FromNode, Id, NodeExtension, ParseContext};

use super::Tint;

#[derive(Clone, PartialEq, Debug)]
pub struct ColorReference {
    pub hue: Id,
    pub tint: Tint,
}

impl ColorReference {
    pub fn with_tint(&self, tint: Tint) -> ColorReference {
        ColorReference {
            hue: self.hue.clone(),
            tint,
        }
    }
}

impl FromNode for ColorReference {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<ColorReference> {
        let arguments: Vec<&KdlValue> = node.arguments().collect();
        let usage = || format!("`{}` needs a hue and a tint, like `\"blue\" 500`", node.node_name());
        let [hue, tint] = arguments[..] else {
            bail!(usage());
        };

        Ok(ColorReference {
            hue: context.parse_id(hue.as_string().with_context(usage)?)?,
            tint: Tint::from_number(tint.as_integer().with_context(usage)? as f64)?,
        })
    }
}
