use anyhow::{Context, bail, ensure};
use kdl::KdlNode;
use xsa_core::scatter::{RandomRange, ScatterModel, ScatterOrientation, ScatterPhysics, ScatterWeight, WeightNoise};

use crate::{FromNode, Id, NodeExtension, ParseContext};

const DEFAULT_CONTRAST: f64 = 1.0;
const MAXIMUM_TILT_DEGREES: f64 = 90.0;

// One model of a scatter: the model's id and how it is placed.
pub struct ScatterModelDefinition {
    pub model: Id,
    pub placement: ScatterModel,
}

impl ScatterModelDefinition {
    fn placement(node: &KdlNode) -> anyhow::Result<ScatterModel> {
        let optional = |name: &str| node.child_nodes().iter().find(|child| child.node_name() == name);

        Ok(ScatterModel {
            physics: ScatterModelDefinition::physics(ScatterModelDefinition::child(node, "physics")?)?,
            orientation: ScatterModelDefinition::orientation(ScatterModelDefinition::child(node, "orientation")?)?,
            scale: ScatterModelDefinition::scale(ScatterModelDefinition::child(node, "scale")?)?,
            sink: optional("sink").map_or(Ok(RandomRange::ZERO), ScatterModelDefinition::sink)?,
            tilt: optional("tilt").map_or(Ok(RandomRange::ZERO), ScatterModelDefinition::tilt)?,
            weight: ScatterModelDefinition::weight(ScatterModelDefinition::child(node, "weight")?)?,
        })
    }

    // A range of two values, the minimum first.
    fn range(node: &KdlNode) -> anyhow::Result<RandomRange> {
        let values = node.number_arguments()?;
        let name = node.node_name();
        ensure!(
            values.len() == 2,
            "`{name}` needs two values: the minimum and the maximum"
        );
        ensure!(
            values[0] <= values[1],
            "`{name}` needs a minimum no larger than its maximum"
        );

        Ok(RandomRange {
            minimum: values[0],
            maximum: values[1],
        })
    }

    fn sink(node: &KdlNode) -> anyhow::Result<RandomRange> {
        let sink = ScatterModelDefinition::range(node)?;
        ensure!(
            sink.minimum >= 0.0 && sink.maximum <= 1.0,
            "`sink` is a share of the model's height, from 0 to 1"
        );

        Ok(sink)
    }

    fn tilt(node: &KdlNode) -> anyhow::Result<RandomRange> {
        let tilt = ScatterModelDefinition::range(node)?;
        ensure!(
            tilt.minimum >= 0.0 && tilt.maximum <= MAXIMUM_TILT_DEGREES,
            "`tilt` is in degrees from upright, from 0 to 90"
        );

        Ok(RandomRange {
            minimum: tilt.minimum.to_radians(),
            maximum: tilt.maximum.to_radians(),
        })
    }

    fn child<'a>(node: &'a KdlNode, name: &str) -> anyhow::Result<&'a KdlNode> {
        node.child_nodes()
            .iter()
            .find(|child| child.node_name() == name)
            .with_context(|| format!("`model` needs a `{name}`"))
    }

    fn physics(node: &KdlNode) -> anyhow::Result<ScatterPhysics> {
        match node.string_argument()? {
            "none" => Ok(ScatterPhysics::None),
            "static" => Ok(ScatterPhysics::Static),
            "dynamic" => Ok(ScatterPhysics::Dynamic),
            other => bail!("unknown physics `{other}`: use none, static or dynamic"),
        }
    }

    fn orientation(node: &KdlNode) -> anyhow::Result<ScatterOrientation> {
        match node.string_argument()? {
            "ground" => Ok(ScatterOrientation::Ground),
            "up" => Ok(ScatterOrientation::Up),
            other => bail!("unknown orientation `{other}`: use ground or up"),
        }
    }

    fn scale(node: &KdlNode) -> anyhow::Result<RandomRange> {
        let scale = ScatterModelDefinition::range(node)?;
        ensure!(scale.minimum > 0.0, "`scale` needs a minimum above 0");

        Ok(scale)
    }

    fn weight(node: &KdlNode) -> anyhow::Result<ScatterWeight> {
        let value = node.number_argument()?;
        ensure!(value > 0.0, "`weight` must be above 0");
        let contrast = node.optional_number_property("contrast")?;
        let noise = match node.optional_number_property("noise")? {
            Some(size) => {
                ensure!(size > 0.0, "`weight` property `noise` must be above 0 meters");
                let contrast = contrast.unwrap_or(DEFAULT_CONTRAST);
                ensure!(contrast >= 1.0, "`weight` property `contrast` must be at least 1");

                Some(WeightNoise { size, contrast })
            }
            None => {
                ensure!(contrast.is_none(), "`weight` property `contrast` needs a `noise` size");
                None
            }
        };

        Ok(ScatterWeight { value, noise })
    }
}

impl FromNode for ScatterModelDefinition {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<ScatterModelDefinition> {
        let model = context.parse_id(node.string_argument()?)?;
        let placement = ScatterModelDefinition::placement(node).with_context(|| format!("model {model}"))?;

        Ok(ScatterModelDefinition { model, placement })
    }
}
