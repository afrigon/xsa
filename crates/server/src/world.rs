mod running_warp;
mod world_options;
mod world_tick;

pub use world_options::WorldOptions;
pub(crate) use world_tick::WorldTick;

use anyhow::{Context, bail};
use running_warp::RunningWarp;
use xsa_core::simulation::Simulation;
use xsa_packs::{Id, PackData, PackStack, SimulationDefinition};
use xsa_proto::session::SessionEnvironment;
use xsa_proto::user::User;
use xsa_proto::world::{PackReference, WorldState};
use xsa_units::{SimulationDuration, SimulationTime, TimeFlow, TimeRate, TimeWarp, WarpPace};

const DEFAULT_NAMESPACE: &str = "base";
// The fastest time runs, past which a frame cannot keep up: 2^17, so doubling from real time reaches it exactly.
pub(crate) const MAXIMUM_TIME_RATE: TimeRate = TimeRate { multiplier: 131_072.0 };

pub struct World {
    simulation_id: Id,
    simulation: Simulation,
    packs: Vec<PackReference>,
    time: SimulationTime,
    rate: TimeRate,
    time_override: Option<TimeRate>,
    warp: Option<RunningWarp>,
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
            time_override: None,
            warp: None,
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

    pub fn time_flow(&self) -> TimeFlow {
        TimeFlow {
            rate: self.rate,
            time_override: self.time_override,
            warp: self.warp.as_ref().map(RunningWarp::warp),
        }
    }

    // The rate time runs at now: the override's, else a warp's, else the configured one.
    pub fn current_rate(&self) -> TimeRate {
        match (self.time_override, &self.warp) {
            (Some(time_override), _) => time_override,
            (None, Some(warp)) => warp.rate(),
            (None, None) => self.rate,
        }
    }

    pub fn state(&self, users: Vec<User>, environment: SessionEnvironment) -> WorldState {
        WorldState {
            simulation: self.simulation_id.to_string(),
            packs: self.packs.clone(),
            time: self.time,
            time_flow: self.time_flow(),
            rate: self.current_rate(),
            environment,
            users,
        }
    }

    // An override holds a warp where it is while time runs at the override's rate.
    pub(crate) fn advance(&mut self, real: SimulationDuration) -> WorldTick {
        let before = self.time;
        let mut flow_changed = false;

        match (&mut self.warp, self.time_override) {
            (Some(warp), None) => {
                self.time = warp.advance(real.seconds);

                if warp.is_finished() {
                    self.warp = None;
                    flow_changed = true;
                }
            }
            _ => self.time = self.time.advanced_by(self.time_flow().effective().scale(real)),
        }

        WorldTick {
            rate: TimeRate {
                multiplier: self.time.seconds_since(before) / real.seconds,
            },
            flow_changed,
        }
    }

    // Every change to time but an override stops a running warp.
    pub(crate) fn set_time(&mut self, time: SimulationTime) {
        self.time = time;
        self.warp = None;
    }

    pub(crate) fn set_rate(&mut self, rate: TimeRate) {
        self.rate = rate;
        self.warp = None;
    }

    // Clearing the override resumes a warp it held, from wherever the override took time.
    pub(crate) fn set_time_override(&mut self, time_override: Option<TimeRate>) {
        if self.time_override.is_some() && time_override.is_none() {
            self.warp = self.warp.as_ref().and_then(|warp| warp.resumed(self.time));
        }

        self.time_override = time_override;
    }

    pub(crate) fn step(&mut self, duration: SimulationDuration) {
        self.time = self.time.advanced_by(duration);
        self.warp = None;
    }

    pub(crate) fn set_warp(&mut self, warp: Option<TimeWarp>) {
        let Some(warp) = warp else {
            self.warp = None;
            return;
        };

        self.warp = match warp.pace {
            WarpPace::Instant => {
                self.time = warp.until;
                None
            }
            WarpPace::Eased { seconds } => Some(RunningWarp::new(warp, self.time, seconds)),
        };
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
