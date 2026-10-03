use super::{BarycenterNode, BodyNode};

pub enum SystemNode {
    Body(Box<BodyNode>),
    Barycenter(Box<BarycenterNode>),
}
