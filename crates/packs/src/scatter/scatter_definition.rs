use anyhow::ensure;
use xsa_core::scatter::Scatter;

use super::ScatterModelDefinition;
use crate::{Document, FromNode, NodeExtension, PackData, ParseContext};

// A scatter as a pack writes it: its spacing and range in meters, and its models by id.
pub struct ScatterDefinition {
    pub spacing: f64,
    pub range: f64,
    pub models: Vec<ScatterModelDefinition>,
}

impl ScatterDefinition {
    // The models keep their order: the placement picks a model by its place in this list.
    pub fn scatter(&self) -> Scatter {
        Scatter {
            spacing: self.spacing,
            range: self.range,
            models: self.models.iter().map(|model| model.placement).collect(),
        }
    }
}

impl PackData for ScatterDefinition {
    const KIND: &'static str = "scatters";

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<ScatterDefinition> {
        let spacing = document.node("spacing")?.number_argument()?;
        let range = document.node("range")?.number_argument()?;
        ensure!(spacing > 0.0, "`spacing` must be above 0 meters");
        ensure!(range > 0.0, "`range` must be above 0 meters");
        let mut models = Vec::new();

        for node in document.nodes() {
            match node.node_name() {
                "spacing" | "range" => {}
                "model" => models.push(ScatterModelDefinition::from_node(node, context)?),
                other => anyhow::bail!("unknown scatter node `{other}`: use `spacing`, `range` or `model`"),
            }
        }

        ensure!(!models.is_empty(), "a scatter needs at least one `model`");

        Ok(ScatterDefinition { spacing, range, models })
    }
}
