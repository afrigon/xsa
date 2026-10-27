use anyhow::ensure;
use glam::Vec3;
use kdl::KdlNode;

use crate::{FromNode, Id, NodeExtension, ParseContext, ScatterDefinition};

const FULL_DENSITY: f64 = 1.0;

// A scatter a biome places: the share of its full density it reaches, and the linear multiplier on its models' base
// colour.
pub struct BiomeScatter {
    pub scatter: Id,
    pub density: f64,
    pub tint: Vec3,
}

impl FromNode for BiomeScatter {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<BiomeScatter> {
        let scatter = context.parse_id(node.string_argument()?)?;
        context.stack.load_data::<ScatterDefinition>(&scatter)?;
        let mut density = FULL_DENSITY;
        let mut tint = Vec3::ONE;

        for child in node.child_nodes() {
            match child.node_name() {
                "density" => density = child.number_argument()?,
                "tint" => {
                    let components = child.number_arguments()?;
                    ensure!(components.len() == 3, "`tint` needs three components: red green blue");
                    tint = Vec3::new(components[0] as f32, components[1] as f32, components[2] as f32);
                }
                other => anyhow::bail!("unknown scatter node `{other}`: use `density` or `tint`"),
            }
        }

        ensure!(
            0.0 < density && density <= FULL_DENSITY,
            "`density` runs above 0 up to 1, the scatter's full density"
        );

        Ok(BiomeScatter { scatter, density, tint })
    }
}
