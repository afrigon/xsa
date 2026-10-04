use super::{BodyNode, SystemNode};
use crate::orbit::OrbitalElements;

pub struct BarycenterNode {
    pub name: String,
    pub orbit: Option<OrbitalElements>,
    pub primary: BodyNode,
    pub secondary: BodyNode,
    pub children: Vec<SystemNode>,
}
