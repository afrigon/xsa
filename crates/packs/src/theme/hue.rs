use anyhow::{Context, bail, ensure};
use kdl::KdlValue;

use crate::{Document, NodeExtension};

use super::{SrgbColor, Tint};

const TINT_NODE: &str = "tint";

pub struct Hue {
    colors: [SrgbColor; Tint::ALL.len()],
}

impl Hue {
    pub fn parse(document: &Document) -> anyhow::Result<Hue> {
        let mut colors: [Option<SrgbColor>; Tint::ALL.len()] = [None; Tint::ALL.len()];

        for node in document.nodes() {
            ensure!(
                node.node_name() == TINT_NODE,
                "unknown node `{}`: a hue lists `tint <number> \"#RRGGBB\"`",
                node.node_name()
            );
            let arguments: Vec<&KdlValue> = node.arguments().collect();
            let [number, color] = arguments[..] else {
                bail!("`tint` needs a number and a color, like `tint 500 \"#1D7AFC\"`");
            };
            let number = number
                .as_integer()
                .context("a tint number must be an integer like 500")?;
            let tint = Tint::from_number(number as f64)?;
            let color = SrgbColor::parse(color.as_string().context("a tint color must be a string")?)?;
            ensure!(
                colors[tint.index()].replace(color).is_none(),
                "tint {} is listed twice",
                tint.number()
            );
        }

        let mut missing = Tint::ALL.into_iter().filter(|tint| colors[tint.index()].is_none());

        if let Some(tint) = missing.next() {
            bail!(
                "tint {} is missing: a hue lists every tint from 100 to 1000",
                tint.number()
            );
        }

        Ok(Hue {
            colors: colors.map(|color| color.expect("every tint was checked")),
        })
    }

    pub fn color(&self, tint: Tint) -> SrgbColor {
        self.colors[tint.index()]
    }
}
