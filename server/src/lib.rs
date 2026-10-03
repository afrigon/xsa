pub mod dedicated;
mod server_user;

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, bail};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use xsa_core::packs::data::SIMULATIONS;
use xsa_core::packs::{Id, PackStack};
use xsa_core::simulation::Simulation;
use xsa_core::time::{self, TICK_RATE_HERTZ};
use xsa_proto::connection::{ClientLink, Connection};
use xsa_proto::messages::{
    ClientFrame, ClientMessage, Outcome, PackReference, Player, PlayerId, Role, ServerEvent, WorldState,
};

pub struct WorldOptions {
    pub packs_directory: PathBuf,
    pub packs: Vec<String>,
    pub simulation: Option<String>,
}

pub struct World {
    simulation: Simulation,
    packs: Vec<PackReference>,
    time: f64,
    rate: f64,
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
            rate: 1.0,
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
    next_player: u32,
}

struct Client {
    link: ClientLink,
    membership: Option<Membership>,
    connected: bool,
}

enum Membership {
    Player { player: Player },
    Console,
}

impl Server {
    pub fn new(world: World, new_clients: UnboundedReceiver<ClientLink>) -> Self {
        Self {
            world,
            new_clients,
            clients: Vec::new(),
            next_player: 0,
        }
    }

    pub fn run(mut self) {
        let tick = Duration::from_secs_f64(1.0 / TICK_RATE_HERTZ);
        let mut next_tick = Instant::now();
        while self.accept_clients() {
            self.advance(tick.as_secs_f64());
            self.handle_frames();
            self.broadcast(&ServerEvent::Tick { time: self.world.time });
            self.remove_disconnected();

            next_tick += tick;
            let now = Instant::now();
            if next_tick > now {
                thread::sleep(next_tick - now);
            } else {
                next_tick = now;
            }
        }
    }

    fn advance(&mut self, seconds: f64) {
        if self.world.rate > 0.0 {
            self.world.time += seconds * self.world.rate;
        }
    }

    fn accept_clients(&mut self) -> bool {
        loop {
            match self.new_clients.try_recv() {
                Ok(link) => self.clients.push(Client {
                    link,
                    membership: None,
                    connected: true,
                }),
                Err(TryRecvError::Empty) => return true,
                Err(TryRecvError::Disconnected) => return !self.clients.is_empty(),
            }
        }
    }

    fn handle_frames(&mut self) {
        for client in 0..self.clients.len() {
            while let Some(frame) = self.receive(client) {
                let outcome = self.handle(client, frame.message);
                self.send(client, ServerEvent::Reply { id: frame.id, outcome });
            }
        }
    }

    fn receive(&mut self, client: usize) -> Option<ClientFrame> {
        let client = &mut self.clients[client];
        if !client.connected {
            return None;
        }
        match client.link.messages.try_recv() {
            Ok(frame) => Some(frame),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                client.connected = false;
                None
            }
        }
    }

    fn handle(&mut self, client: usize, message: ClientMessage) -> Outcome {
        let needs_membership = !matches!(message, ClientMessage::Join { .. } | ClientMessage::Leave);
        if needs_membership && self.clients[client].membership.is_none() {
            return denied("join the simulation first");
        }
        match message {
            ClientMessage::Join { role } => self.join(client, role),
            ClientMessage::Leave => {
                self.clients[client].connected = false;
                Outcome::Accepted
            }
            ClientMessage::SetTimeRate { rate } => self.set_time_rate(rate),
            ClientMessage::SetTime { time } => self.set_time(time),
            ClientMessage::StepTime { seconds } => self.step_time(seconds),
        }
    }

    fn set_time_rate(&mut self, rate: f64) -> Outcome {
        if !(rate.is_finite() && rate >= 0.0) {
            return denied("the time rate must be a finite number of at least 0");
        }
        self.world.rate = rate;
        self.broadcast_time();
        Outcome::Accepted
    }

    fn set_time(&mut self, time: f64) -> Outcome {
        if !time.is_finite() {
            return denied("the time must be finite");
        }
        self.world.time = time;
        self.broadcast_time();
        Outcome::Accepted
    }

    fn step_time(&mut self, seconds: f64) -> Outcome {
        if !(seconds.is_finite() && seconds > 0.0) {
            return denied("the step must be a finite duration greater than 0");
        }
        self.world.time += seconds;
        self.broadcast_time();
        Outcome::Accepted
    }

    fn join(&mut self, client: usize, role: Role) -> Outcome {
        if self.clients[client].membership.is_some() {
            return denied("already joined");
        }
        let membership = match role {
            Role::Player { name } => {
                let player = Player {
                    id: PlayerId {
                        value: self.next_player,
                    },
                    name,
                };
                self.next_player += 1;
                Membership::Player { player }
            }
            Role::Console => Membership::Console,
        };
        let joined_player = match &membership {
            Membership::Player { player } => Some(player.clone()),
            Membership::Console => None,
        };
        self.clients[client].membership = Some(membership);
        let state = WorldState {
            simulation: self.world.simulation.id().to_string(),
            packs: self.world.packs.clone(),
            time: self.world.time,
            rate: self.world.rate,
            players: self.players(),
        };
        self.send(client, ServerEvent::JoinAccepted { state });
        if let Some(player) = joined_player {
            self.broadcast_except(Some(client), &ServerEvent::PlayerJoined { player });
        }
        Outcome::Accepted
    }

    fn players(&self) -> Vec<Player> {
        self.clients
            .iter()
            .filter_map(|client| match &client.membership {
                Some(Membership::Player { player }) => Some(player.clone()),
                _ => None,
            })
            .collect()
    }

    fn broadcast_time(&mut self) {
        self.broadcast(&ServerEvent::TimeChanged {
            time: self.world.time,
            rate: self.world.rate,
        });
    }

    fn send(&mut self, client: usize, event: ServerEvent) {
        let client = &mut self.clients[client];
        if client.connected && client.link.events.send(event).is_err() {
            client.connected = false;
        }
    }

    fn broadcast(&mut self, event: &ServerEvent) {
        self.broadcast_except(None, event);
    }

    fn broadcast_except(&mut self, excluded: Option<usize>, event: &ServerEvent) {
        for (index, client) in self.clients.iter_mut().enumerate() {
            if Some(index) != excluded
                && client.connected
                && client.membership.is_some()
                && client.link.events.send(event.clone()).is_err()
            {
                client.connected = false;
            }
        }
    }

    fn remove_disconnected(&mut self) {
        while let Some(index) = self.clients.iter().position(|client| !client.connected) {
            let client = self.clients.remove(index);
            if let Some(Membership::Player { player }) = client.membership {
                self.broadcast(&ServerEvent::PlayerLeft { player: player.id });
            }
        }
    }
}

fn denied(reason: &str) -> Outcome {
    Outcome::Denied {
        reason: reason.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use xsa_proto::messages::MessageId;

    struct Harness {
        server: Server,
        next_id: u64,
    }

    impl Harness {
        fn new() -> Self {
            let world = World::load(&WorldOptions {
                packs_directory: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../packs")),
                packs: vec!["base".to_string(), "system-solar".to_string()],
                simulation: None,
            })
            .unwrap();
            let (_new_clients, receiver) = unbounded_channel();
            Self {
                server: Server::new(world, receiver),
                next_id: 0,
            }
        }

        fn connect(&mut self) -> Connection {
            let (connection, link) = Connection::local();
            self.server.clients.push(Client {
                link,
                membership: None,
                connected: true,
            });
            connection
        }

        fn send(&mut self, connection: &Connection, message: ClientMessage) -> MessageId {
            let id = MessageId { value: self.next_id };
            self.next_id += 1;
            connection.send(ClientFrame { id, message }).unwrap();
            self.server.handle_frames();
            self.server.remove_disconnected();
            id
        }
    }

    fn drain(connection: &mut Connection) -> Vec<ServerEvent> {
        let mut events = Vec::new();
        while let Ok(Some(event)) = connection.poll() {
            events.push(event);
        }
        events
    }

    fn player(name: &str) -> ClientMessage {
        ClientMessage::Join {
            role: Role::Player { name: name.to_string() },
        }
    }

    #[test]
    fn join_is_answered_with_the_state_then_a_reply() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        let id = harness.send(&connection, player("ada"));
        let events = drain(&mut connection);
        assert!(
            matches!(&events[0], ServerEvent::JoinAccepted { state } if state.rate == 1.0 && state.players.len() == 1)
        );
        assert_eq!(
            events[1],
            ServerEvent::Reply {
                id,
                outcome: Outcome::Accepted
            }
        );
    }

    #[test]
    fn a_time_change_is_broadcast_before_the_reply() {
        let mut harness = Harness::new();
        let mut console = harness.connect();
        let mut observer = harness.connect();
        harness.send(&console, ClientMessage::Join { role: Role::Console });
        harness.send(&observer, player("ada"));
        drain(&mut console);
        drain(&mut observer);

        let id = harness.send(&console, ClientMessage::SetTimeRate { rate: 100.0 });
        let time = harness.server.world.time;
        assert_eq!(
            drain(&mut console),
            vec![
                ServerEvent::TimeChanged { time, rate: 100.0 },
                ServerEvent::Reply {
                    id,
                    outcome: Outcome::Accepted
                },
            ]
        );
        assert_eq!(
            drain(&mut observer),
            vec![ServerEvent::TimeChanged { time, rate: 100.0 }]
        );
    }

    #[test]
    fn stepping_advances_time_by_the_step() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, ClientMessage::Join { role: Role::Console });
        let before = harness.server.world.time;
        harness.send(&connection, ClientMessage::StepTime { seconds: 10.0 });
        assert_eq!(harness.server.world.time, before + 10.0);
    }

    #[test]
    fn a_paused_world_does_not_advance() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, ClientMessage::Join { role: Role::Console });
        harness.send(&connection, ClientMessage::SetTimeRate { rate: 0.0 });
        let before = harness.server.world.time;
        harness.server.advance(1.0);
        assert_eq!(harness.server.world.time, before);
    }

    #[test]
    fn invalid_values_are_denied_without_changing_the_world() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(&connection, ClientMessage::Join { role: Role::Console });
        drain(&mut connection);
        for message in [
            ClientMessage::SetTimeRate { rate: -1.0 },
            ClientMessage::SetTimeRate { rate: f64::NAN },
            ClientMessage::SetTime { time: f64::INFINITY },
            ClientMessage::StepTime { seconds: 0.0 },
        ] {
            harness.send(&connection, message);
            let events = drain(&mut connection);
            assert!(
                matches!(
                    events.as_slice(),
                    [ServerEvent::Reply {
                        outcome: Outcome::Denied { .. },
                        ..
                    }]
                ),
                "{events:?}"
            );
        }
        assert_eq!(harness.server.world.rate, 1.0);
    }

    #[test]
    fn messages_before_joining_are_denied() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(&connection, ClientMessage::SetTimeRate { rate: 2.0 });
        assert!(matches!(
            drain(&mut connection).as_slice(),
            [ServerEvent::Reply {
                outcome: Outcome::Denied { .. },
                ..
            }]
        ));
    }

    #[test]
    fn players_joining_and_leaving_are_announced_to_others() {
        let mut harness = Harness::new();
        let mut first = harness.connect();
        let second = harness.connect();
        let console = harness.connect();
        harness.send(&first, player("ada"));
        drain(&mut first);

        harness.send(&console, ClientMessage::Join { role: Role::Console });
        assert!(drain(&mut first).is_empty());

        harness.send(&second, player("grace"));
        let joined = drain(&mut first);
        let [ServerEvent::PlayerJoined { player }] = joined.as_slice() else {
            panic!("{joined:?}");
        };
        assert_eq!(player.name, "grace");

        let grace = player.id;
        harness.send(&second, ClientMessage::Leave);
        assert_eq!(drain(&mut first), vec![ServerEvent::PlayerLeft { player: grace }]);
    }
}
