use std::path::PathBuf;

use anyhow::{bail, ensure};

use crate::{Document, NodeExtension, ParseContext};

const FONT_NODE: &str = "font";

pub struct Typeface {
    pub fonts: Vec<PathBuf>,
}

impl Typeface {
    pub fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<Typeface> {
        let mut fonts = Vec::new();

        for node in document.nodes() {
            if node.node_name() != FONT_NODE {
                bail!(
                    "unknown node `{}`: a typeface lists `font \"<font id>\"`",
                    node.node_name()
                );
            }

            let id = context.parse_id(node.string_argument()?)?;
            fonts.push(context.stack.font(&id)?.to_path_buf());
        }

        ensure!(!fonts.is_empty(), "a typeface needs at least one `font`");

        Ok(Typeface { fonts })
    }
}
