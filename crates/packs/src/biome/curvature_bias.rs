use crate::NodeExtension;

// How the ground's shape moves its class: convex ground leans steep, concave ground leans flat, and ground below its
// surroundings leans flat. Radians of slope per unit of each measure: curvature is the change in slope across one
// texel of the heights, relief the height above the ground two levels coarser over that level's texel spacing.
#[derive(Clone, Copy, Debug)]
pub struct CurvatureBias {
    pub convex: f32,
    pub concave: f32,
    pub relief: f32,
}

impl CurvatureBias {
    pub(super) fn parse(node: &kdl::KdlNode) -> anyhow::Result<CurvatureBias> {
        Ok(CurvatureBias {
            convex: (node.number_property("convex")? as f32).to_radians(),
            concave: (node.number_property("concave")? as f32).to_radians(),
            relief: (node.number_property("relief")? as f32).to_radians(),
        })
    }
}
