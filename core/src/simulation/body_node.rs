use super::{BodyDescription, Spin, SystemNode};
use crate::orbit::OrbitalElements;

pub struct BodyNode {
    pub body: BodyDescription,
    pub orbit: Option<OrbitalElements>,
    pub spin: Spin,
    pub children: Vec<SystemNode>,
}
