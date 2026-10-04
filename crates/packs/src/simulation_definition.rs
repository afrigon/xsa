use xsa_core::simulation::{BodyId, Simulation, SystemTree};

use crate::{Document, Id, NodeExtension, PackData, PackStack, ParseContext};

pub struct SimulationDefinition {
    pub id: Id,
    pub system: Id,
    pub spawn: Option<Id>,
}

impl SimulationDefinition {
    pub fn build(&self, stack: &PackStack) -> anyhow::Result<Simulation> {
        let system = stack.load_data::<SystemTree>(&self.system)?;
        let spawn = self.spawn.as_ref().map(BodyId::from);

        Simulation::new(&system, spawn.as_ref())
    }
}

impl PackData for SimulationDefinition {
    const KIND: &'static str = "simulations";

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<SimulationDefinition> {
        Ok(SimulationDefinition {
            id: context.id.clone(),
            system: context.parse_id(document.node("system")?.string_argument()?)?,
            spawn: document
                .optional_node("spawn")
                .map(|node| context.parse_id(node.string_argument()?))
                .transpose()?,
        })
    }
}
