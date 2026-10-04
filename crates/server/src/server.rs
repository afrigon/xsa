mod client;
mod handlers;
mod membership;
mod message_handler;

use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use xsa_proto::connection::{ClientLink, Connection};
use xsa_proto::event::{Outcome, Player, PlayerLeft, Reply, ServerEvent, Tick, TimeChanged};
use xsa_proto::message::{ClientFrame, ClientMessage};
use xsa_units::{SimulationDuration, TICK_RATE_HERTZ};

use crate::World;
use client::Client;
use membership::Membership;
use message_handler::MessageHandler;

pub struct Server {
    world: World,
    new_clients: UnboundedReceiver<ClientLink>,
    clients: Vec<Client>,
    next_player: u32,
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

    pub fn start_local(world: World) -> anyhow::Result<Connection> {
        let local = Connection::local();
        let (new_clients, receiver) = unbounded_channel();
        new_clients
            .send(local.link)
            .expect("the receiver is alive until the server starts");
        thread::Builder::new()
            .name("integrated server".into())
            .spawn(move || Server::new(world, receiver).run())?;

        Ok(local.connection)
    }

    pub fn run(mut self) {
        let tick = Duration::from_secs_f64(1.0 / TICK_RATE_HERTZ);
        let mut next_tick = Instant::now();

        while self.accept_clients() {
            self.world.advance(SimulationDuration {
                seconds: tick.as_secs_f64(),
            });
            self.handle_frames();
            self.broadcast(&ServerEvent::Tick(Tick {
                time: self.world.time(),
            }));
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

    fn accept_clients(&mut self) -> bool {
        loop {
            match self.new_clients.try_recv() {
                Ok(link) => self.clients.push(Client::new(link)),
                Err(TryRecvError::Empty) => return true,
                Err(TryRecvError::Disconnected) => return !self.clients.is_empty(),
            }
        }
    }

    fn handle_frames(&mut self) {
        for client in 0..self.clients.len() {
            while let Some(frame) = self.receive(client) {
                let outcome = self.handle(client, frame.message);
                self.send(client, ServerEvent::Reply(Reply { id: frame.id, outcome }));
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
        let needs_membership = !matches!(message, ClientMessage::Join(_) | ClientMessage::Leave(_));

        if needs_membership && self.clients[client].membership.is_none() {
            return Outcome::denied("join the simulation first");
        }

        match message {
            ClientMessage::Join(message) => message.handle(self, client),
            ClientMessage::Leave(message) => message.handle(self, client),
            ClientMessage::SetTimeRate(message) => message.handle(self, client),
            ClientMessage::SetTime(message) => message.handle(self, client),
            ClientMessage::StepTime(message) => message.handle(self, client),
        }
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
        self.broadcast(&ServerEvent::TimeChanged(TimeChanged {
            time: self.world.time(),
            rate: self.world.rate(),
        }));
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
                self.broadcast(&ServerEvent::PlayerLeft(PlayerLeft { player: player.id }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use xsa_proto::message::{Join, Leave, MessageId, Role, SetTime, SetTimeRate, StepTime};
    use xsa_units::{SimulationTime, TimeRate};

    use super::*;
    use crate::WorldOptions;

    struct Harness {
        server: Server,
        next_id: u64,
    }

    impl Harness {
        fn new() -> Self {
            let world = World::load(&WorldOptions {
                packs_directory: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")),
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
            let local = Connection::local();
            self.server.clients.push(Client::new(local.link));

            local.connection
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
        ClientMessage::Join(Join {
            role: Role::Player { name: name.to_string() },
        })
    }

    fn server_role() -> ClientMessage {
        ClientMessage::Join(Join { role: Role::Server })
    }

    fn accepted(id: MessageId) -> ServerEvent {
        ServerEvent::Reply(Reply {
            id,
            outcome: Outcome::Accepted,
        })
    }

    fn is_denied(events: &[ServerEvent]) -> bool {
        matches!(
            events,
            [ServerEvent::Reply(Reply {
                outcome: Outcome::Denied { .. },
                ..
            })]
        )
    }

    #[test]
    fn join_is_answered_with_the_state_then_a_reply() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        let id = harness.send(&connection, player("ada"));
        let events = drain(&mut connection);
        assert!(matches!(
            &events[0],
            ServerEvent::JoinAccepted(accepted)
                if accepted.state.rate == TimeRate::REAL_TIME && accepted.state.players.len() == 1
        ));
        assert_eq!(events[1], accepted(id));
    }

    #[test]
    fn a_time_change_is_broadcast_before_the_reply() {
        let mut harness = Harness::new();
        let mut server_user = harness.connect();
        let mut observer = harness.connect();
        harness.send(&server_user, server_role());
        harness.send(&observer, player("ada"));
        drain(&mut server_user);
        drain(&mut observer);

        let rate = TimeRate { multiplier: 100.0 };
        let id = harness.send(&server_user, ClientMessage::SetTimeRate(SetTimeRate { rate }));
        let changed = ServerEvent::TimeChanged(TimeChanged {
            time: harness.server.world.time(),
            rate,
        });
        assert_eq!(drain(&mut server_user), vec![changed.clone(), accepted(id)]);
        assert_eq!(drain(&mut observer), vec![changed]);
    }

    #[test]
    fn stepping_advances_time_by_the_step() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, server_role());
        let before = harness.server.world.time();
        let duration = SimulationDuration { seconds: 10.0 };
        harness.send(&connection, ClientMessage::StepTime(StepTime { duration }));
        assert_eq!(harness.server.world.time(), before.advanced_by(duration));
    }

    #[test]
    fn a_paused_world_does_not_advance() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, server_role());
        harness.send(
            &connection,
            ClientMessage::SetTimeRate(SetTimeRate { rate: TimeRate::PAUSED }),
        );
        let before = harness.server.world.time();
        harness.server.world.advance(SimulationDuration { seconds: 1.0 });
        assert_eq!(harness.server.world.time(), before);
    }

    #[test]
    fn invalid_values_are_denied_without_changing_the_world() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(&connection, server_role());
        drain(&mut connection);

        for message in [
            ClientMessage::SetTimeRate(SetTimeRate {
                rate: TimeRate { multiplier: -1.0 },
            }),
            ClientMessage::SetTimeRate(SetTimeRate {
                rate: TimeRate { multiplier: f64::NAN },
            }),
            ClientMessage::SetTime(SetTime {
                time: SimulationTime { seconds: f64::INFINITY },
            }),
            ClientMessage::StepTime(StepTime {
                duration: SimulationDuration { seconds: 0.0 },
            }),
        ] {
            harness.send(&connection, message);
            let events = drain(&mut connection);
            assert!(is_denied(&events), "{events:?}");
        }

        assert_eq!(harness.server.world.rate(), TimeRate::REAL_TIME);
    }

    #[test]
    fn messages_before_joining_are_denied() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(
            &connection,
            ClientMessage::SetTimeRate(SetTimeRate {
                rate: TimeRate { multiplier: 2.0 },
            }),
        );
        assert!(is_denied(&drain(&mut connection)));
    }

    #[test]
    fn players_joining_and_leaving_are_announced_to_others() {
        let mut harness = Harness::new();
        let mut first = harness.connect();
        let second = harness.connect();
        let server_user = harness.connect();
        harness.send(&first, player("ada"));
        drain(&mut first);

        harness.send(&server_user, server_role());
        assert!(drain(&mut first).is_empty());

        harness.send(&second, player("grace"));
        let joined = drain(&mut first);
        let [ServerEvent::PlayerJoined(joined)] = joined.as_slice() else {
            panic!("{joined:?}");
        };
        assert_eq!(joined.player.name, "grace");

        let grace = joined.player.id;
        harness.send(&second, ClientMessage::Leave(Leave));
        assert_eq!(
            drain(&mut first),
            vec![ServerEvent::PlayerLeft(PlayerLeft { player: grace })]
        );
    }
}
