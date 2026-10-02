pub mod dedicated;

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, bail};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use xsa_core::packs::data::SIMULATIONS;
use xsa_core::packs::{Id, PackStack};
use xsa_core::simulation::Simulation;
use xsa_core::time;
use xsa_proto::connection::{ClientLink, Connection};
use xsa_proto::messages::{ClientMessage, PackReference, ServerEvent};

const TICK_RATE_HERTZ: f64 = 20.0;

pub struct WorldOptions {
    pub packs_directory: PathBuf,
    pub packs: Vec<String>,
    pub simulation: Option<String>,
}

pub struct World {
    simulation: Simulation,
    packs: Vec<PackReference>,
    time: f64,
}

impl World {
    pub fn load(options: &WorldOptions) -> anyhow::Result<World> {
        let stack = PackStack::load(&options.packs_directory, &options.packs)?;
        let simulation_id = match &options.simulation {
            Some(id) => Id::parse(id, "base")?,
            None => only_simulation(&stack)?,
        };
        let simulation =
            Simulation::load(&stack, &simulation_id).with_context(|| format!("loading simulation {simulation_id}"))?;
        let packs = stack
            .manifests()
            .iter()
            .map(|manifest| PackReference {
                id: manifest.id.clone(),
                version: manifest.version.to_string(),
            })
            .collect();
        Ok(World {
            simulation,
            packs,
            time: time::now()?,
        })
    }

    pub fn simulation(&self) -> &Simulation {
        &self.simulation
    }
}

fn only_simulation(stack: &PackStack) -> anyhow::Result<Id> {
    let mut simulations = stack.data_ids(SIMULATIONS);
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

pub struct Server {
    world: World,
    new_clients: UnboundedReceiver<ClientLink>,
    clients: Vec<Client>,
}

struct Client {
    link: ClientLink,
    joined: bool,
}

impl Server {
    pub fn new(world: World, new_clients: UnboundedReceiver<ClientLink>) -> Self {
        Self {
            world,
            new_clients,
            clients: Vec::new(),
        }
    }

    pub fn run(mut self) {
        let tick = Duration::from_secs_f64(1.0 / TICK_RATE_HERTZ);
        let mut next_tick = Instant::now();
        while self.accept_clients() {
            self.handle_messages();
            self.world.time += tick.as_secs_f64();
            self.broadcast(&ServerEvent::Tick { time: self.world.time });

            next_tick += tick;
            let now = Instant::now();
            if next_tick > now {
                thread::sleep(next_tick - now);
            } else {
                next_tick = now;
            }
        }
    }

    fn accept_clients(&mut self) -> bool {
        loop {
            match self.new_clients.try_recv() {
                Ok(link) => self.clients.push(Client { link, joined: false }),
                Err(TryRecvError::Empty) => return true,
                Err(TryRecvError::Disconnected) => return !self.clients.is_empty(),
            }
        }
    }

    fn handle_messages(&mut self) {
        let world = &self.world;
        self.clients.retain_mut(|client| {
            loop {
                match client.link.messages.try_recv() {
                    Ok(ClientMessage::Join) => {
                        client.joined = true;
                        let joined = ServerEvent::Joined {
                            simulation: world.simulation.id().to_string(),
                            packs: world.packs.clone(),
                            time: world.time,
                        };
                        if client.link.events.send(joined).is_err() {
                            return false;
                        }
                    }
                    Ok(ClientMessage::Leave) | Err(TryRecvError::Disconnected) => return false,
                    Err(TryRecvError::Empty) => return true,
                }
            }
        });
    }

    fn broadcast(&mut self, event: &ServerEvent) {
        self.clients
            .retain(|client| !client.joined || client.link.events.send(event.clone()).is_ok());
    }
}

pub fn start_local(world: World) -> anyhow::Result<Connection> {
    let (connection, link) = Connection::local();
    let (new_clients, receiver) = unbounded_channel();
    new_clients
        .send(link)
        .expect("the receiver is alive until the server starts");
    thread::Builder::new()
        .name("integrated server".into())
        .spawn(move || Server::new(world, receiver).run())?;
    Ok(connection)
}
