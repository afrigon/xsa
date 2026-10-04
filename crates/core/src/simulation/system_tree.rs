use xsa_units::SimulationTime;

use super::SystemNode;

pub struct SystemTree {
    pub epoch: SimulationTime,
    pub root: SystemNode,
}
