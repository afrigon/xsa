mod world_options;

pub use world_options::WorldOptions;

use anyhow::{Context, bail};
use xsa_core::simulation::Simulation;
use xsa_packs::{Id, PackData, PackStack, SimulationDefinition};
use xsa_proto::event::{PackReference, Player, WorldState};
use xsa_units::{SimulationDuration, SimulationTime, TimeRate};

const DEFAULT_NAMESPACE: &str = "base";

pub struct World {
    simulation_id: Id,
    simulation: Simulation,
    packs: Vec<PackReference>,
    time: SimulationTime,
    rate: TimeRate,
}

impl World {
    pub fn load(options: &WorldOptions) -> anyhow::Result<World> {
        let stack = PackStack::load(&options.packs_directory, &options.packs)?;
        let simulation_id = match &options.simulation {
            Some(id) => Id::parse(id, DEFAULT_NAMESPACE)?,
            None => World::only_simulation(&stack)?,
        };
        let simulation = stack
            .load_data::<SimulationDefinition>(&simulation_id)
            .and_then(|definition| definition.build(&stack))
            .with_context(|| format!("loading simulation {simulation_id}"))?;
        let packs = stack
            .manifests()
            .iter()
            .map(|manifest| PackReference {
                id: manifest.id.clone(),
                version: manifest.version.to_string(),
            })
            .collect();

        Ok(World {
            simulation_id,
            simulation,
            packs,
            time: SimulationTime::now()?,
            rate: TimeRate::REAL_TIME,
        })
    }

    pub fn simulation_id(&self) -> &Id {
        &self.simulation_id
    }

    pub fn simulation(&self) -> &Simulation {
        &self.simulation
    }

    pub fn time(&self) -> SimulationTime {
        self.time
    }

    pub fn rate(&self) -> TimeRate {
        self.rate
    }

    pub fn state(&self, players: Vec<Player>) -> WorldState {
        WorldState {
            simulation: self.simulation_id.to_string(),
            packs: self.packs.clone(),
            time: self.time,
            rate: self.rate,
            players,
        }
    }

    pub(crate) fn advance(&mut self, real: SimulationDuration) {
        self.time = self.time.advanced_by(self.rate.scale(real));
    }

    pub(crate) fn set_time(&mut self, time: SimulationTime) {
        self.time = time;
    }

    pub(crate) fn set_rate(&mut self, rate: TimeRate) {
        self.rate = rate;
    }

    pub(crate) fn step(&mut self, duration: SimulationDuration) {
        self.time = self.time.advanced_by(duration);
    }

    fn only_simulation(stack: &PackStack) -> anyhow::Result<Id> {
        let mut simulations = stack.data_ids(SimulationDefinition::KIND);

        match simulations.len() {
            0 => bail!("no star system is installed: add a pack that defines a simulation"),
            1 => Ok(simulations.remove(0)),
            _ => {
                let names: Vec<String> = simulations.iter().map(Id::to_string).collect();
                bail!(
                    "several simulations are installed, pick one with --simulation: {}",
                    names.join(", ")
                )
            }
        }
    }
}
