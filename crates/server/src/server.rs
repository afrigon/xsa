mod client;
mod handlers;
mod membership;
mod message_handler;

use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use xsa_proto::event::{Event, Outcome, Reply, TickEvent, TimeChangedEvent, UserLeftEvent};
use xsa_proto::frame::ClientFrame;
use xsa_proto::network::{ClientLink, Connection};
use xsa_proto::request::Request;
use xsa_proto::session::SessionEnvironment;
use xsa_proto::user::User;
use xsa_units::{SimulationDuration, TICK_RATE_HERTZ};

use crate::World;
use client::Client;
use membership::Membership;
use message_handler::MessageHandler;

pub struct Server {
    world: World,
    environment: SessionEnvironment,
    new_clients: UnboundedReceiver<ClientLink>,
    clients: Vec<Client>,
    next_user: u32,
}

impl Server {
    pub fn new(world: World, new_clients: UnboundedReceiver<ClientLink>, environment: SessionEnvironment) -> Self {
        Self {
            world,
            environment,
            new_clients,
            clients: Vec::new(),
            next_user: 0,
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
            .spawn(move || Server::new(world, receiver, SessionEnvironment::Singleplayer).run())?;

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
            self.broadcast(&Event::Tick(TickEvent {
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
                let outcome = self.handle(client, frame.request);
                self.send(client, Event::Reply(Reply { id: frame.id, outcome }));
            }
        }
    }

    fn receive(&mut self, client: usize) -> Option<ClientFrame> {
        let client = &mut self.clients[client];

        if !client.connected {
            return None;
        }

        match client.link.requests.try_recv() {
            Ok(frame) => Some(frame),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                client.connected = false;
                None
            }
        }
    }

    fn handle(&mut self, client: usize, request: Request) -> Outcome {
        let needs_membership = !matches!(request, Request::Join(_) | Request::Leave(_));

        if needs_membership && self.clients[client].membership.is_none() {
            return Outcome::denied("join the simulation first");
        }

        if let Err(reason) = request.check_session_environment(self.environment) {
            return Outcome::denied(reason);
        }

        match request {
            Request::Join(request) => request.handle(self, client),
            Request::Leave(request) => request.handle(self, client),
            Request::SetTimeRate(request) => request.handle(self, client),
            Request::SetTimeOverride(request) => request.handle(self, client),
            Request::SetTime(request) => request.handle(self, client),
            Request::StepTime(request) => request.handle(self, client),
        }
    }

    fn users(&self) -> Vec<User> {
        self.clients
            .iter()
            .filter_map(|client| match &client.membership {
                Some(Membership::User { user }) => Some(user.clone()),
                _ => None,
            })
            .collect()
    }

    fn broadcast_time(&mut self) {
        self.broadcast(&Event::TimeChanged(TimeChangedEvent {
            time: self.world.time(),
            time_flow: self.world.time_flow(),
        }));
    }

    fn send(&mut self, client: usize, event: Event) {
        let client = &mut self.clients[client];

        if client.connected && client.link.events.send(event).is_err() {
            client.connected = false;
        }
    }

    fn broadcast(&mut self, event: &Event) {
        self.broadcast_except(None, event);
    }

    fn broadcast_except(&mut self, excluded: Option<usize>, event: &Event) {
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

            if let Some(Membership::User { user }) = client.membership {
                self.broadcast(&Event::UserLeft(UserLeftEvent { user: user.id }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use xsa_proto::request::{
        JoinRequest, LeaveRequest, RequestId, Role, SetTimeOverrideRequest, SetTimeRateRequest, SetTimeRequest,
        StepTimeRequest,
    };
    use xsa_units::{SimulationTime, TimeFlow, TimeRate, TimeRateChange};

    use super::*;
    use crate::WorldOptions;

    struct Harness {
        server: Server,
        next_id: u64,
    }

    impl Harness {
        fn new() -> Self {
            Self::in_environment(SessionEnvironment::Singleplayer)
        }

        fn in_environment(environment: SessionEnvironment) -> Self {
            let world = World::load(&WorldOptions {
                packs_directory: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")),
                packs: vec![
                    "base".to_string(),
                    "base-surfaces".to_string(),
                    "system-solar".to_string(),
                ],
                simulation: None,
            })
            .unwrap();
            let (_new_clients, receiver) = unbounded_channel();

            Self {
                server: Server::new(world, receiver, environment),
                next_id: 0,
            }
        }

        fn connect(&mut self) -> Connection {
            let local = Connection::local();
            self.server.clients.push(Client::new(local.link));

            local.connection
        }

        fn send(&mut self, connection: &Connection, request: Request) -> RequestId {
            let id = RequestId { value: self.next_id };
            self.next_id += 1;
            connection.send(ClientFrame { id, request }).unwrap();
            self.server.handle_frames();
            self.server.remove_disconnected();

            id
        }
    }

    fn drain(connection: &mut Connection) -> Vec<Event> {
        let mut events = Vec::new();

        while let Ok(Some(event)) = connection.poll() {
            events.push(event);
        }

        events
    }

    fn user(name: &str) -> Request {
        Request::Join(JoinRequest {
            role: Role::Player { name: name.to_string() },
        })
    }

    fn server_role() -> Request {
        Request::Join(JoinRequest { role: Role::Server })
    }

    fn accepted(id: RequestId) -> Event {
        Event::Reply(Reply {
            id,
            outcome: Outcome::Accepted,
        })
    }

    fn is_denied(events: &[Event]) -> bool {
        matches!(
            events,
            [Event::Reply(Reply {
                outcome: Outcome::Denied { .. },
                ..
            })]
        )
    }

    #[test]
    fn join_is_answered_with_the_state_then_a_reply() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        let id = harness.send(&connection, user("ada"));
        let events = drain(&mut connection);
        assert!(matches!(
            &events[0],
            Event::JoinAccepted(accepted)
                if accepted.state.time_flow == TimeFlow::REAL_TIME && accepted.state.users.len() == 1
        ));
        assert_eq!(events[1], accepted(id));
    }

    #[test]
    fn a_time_change_is_broadcast_before_the_reply() {
        let mut harness = Harness::new();
        let mut server_user = harness.connect();
        let mut observer = harness.connect();
        harness.send(&server_user, server_role());
        harness.send(&observer, user("ada"));
        drain(&mut server_user);
        drain(&mut observer);

        let rate = TimeRate { multiplier: 100.0 };
        let id = harness.send(
            &server_user,
            Request::SetTimeRate(SetTimeRateRequest {
                change: TimeRateChange::To(rate),
            }),
        );
        let changed = Event::TimeChanged(TimeChangedEvent {
            time: harness.server.world.time(),
            time_flow: TimeFlow {
                rate,
                time_override: None,
            },
        });
        assert_eq!(drain(&mut server_user), vec![changed.clone(), accepted(id)]);
        assert_eq!(drain(&mut observer), vec![changed]);
    }

    #[test]
    fn faster_and_slower_change_the_world_rate() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, server_role());

        for (change, multiplier) in [
            (TimeRateChange::Faster, 2.0),
            (TimeRateChange::Faster, 4.0),
            (TimeRateChange::Slower, 2.0),
            (TimeRateChange::Slower, 1.0),
            (TimeRateChange::Slower, 0.5),
            (TimeRateChange::Slower, 0.25),
            (TimeRateChange::Slower, 0.1),
            (TimeRateChange::Slower, 0.0),
        ] {
            harness.send(&connection, Request::SetTimeRate(SetTimeRateRequest { change }));
            assert_eq!(harness.server.world.time_flow().rate, TimeRate { multiplier });
        }
    }

    #[test]
    fn stepping_advances_time_by_the_step() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, server_role());
        let before = harness.server.world.time();
        let duration = SimulationDuration { seconds: 10.0 };
        harness.send(&connection, Request::StepTime(StepTimeRequest { duration }));
        assert_eq!(harness.server.world.time(), before.advanced_by(duration));
    }

    #[test]
    fn a_paused_world_does_not_advance() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, server_role());
        harness.send(
            &connection,
            Request::SetTimeRate(SetTimeRateRequest {
                change: TimeRateChange::To(TimeRate::PAUSED),
            }),
        );
        let before = harness.server.world.time();
        harness.server.world.advance(SimulationDuration { seconds: 1.0 });
        assert_eq!(harness.server.world.time(), before);
    }

    #[test]
    fn an_override_stops_time_and_keeps_the_rate() {
        let mut harness = Harness::new();
        let connection = harness.connect();
        harness.send(&connection, server_role());
        let rate = TimeRate { multiplier: 4.0 };
        harness.send(
            &connection,
            Request::SetTimeRate(SetTimeRateRequest {
                change: TimeRateChange::To(rate),
            }),
        );
        harness.send(
            &connection,
            Request::SetTimeOverride(SetTimeOverrideRequest {
                rate: Some(TimeRate::PAUSED),
            }),
        );
        let before = harness.server.world.time();
        harness.server.world.advance(SimulationDuration { seconds: 1.0 });
        assert_eq!(harness.server.world.time(), before);
        assert_eq!(harness.server.world.time_flow().rate, rate);

        harness.send(
            &connection,
            Request::SetTimeOverride(SetTimeOverrideRequest { rate: None }),
        );
        assert_eq!(
            harness.server.world.time_flow(),
            TimeFlow {
                rate,
                time_override: None,
            }
        );
    }

    #[test]
    fn a_dedicated_server_denies_a_time_override() {
        let mut harness = Harness::in_environment(SessionEnvironment::Dedicated);
        let mut connection = harness.connect();
        harness.send(&connection, server_role());
        drain(&mut connection);

        harness.send(
            &connection,
            Request::SetTimeOverride(SetTimeOverrideRequest {
                rate: Some(TimeRate::PAUSED),
            }),
        );
        assert!(is_denied(&drain(&mut connection)));
        assert_eq!(harness.server.world.time_flow(), TimeFlow::REAL_TIME);
    }

    #[test]
    fn invalid_values_are_denied_without_changing_the_world() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(&connection, server_role());
        drain(&mut connection);

        for request in [
            Request::SetTimeRate(SetTimeRateRequest {
                change: TimeRateChange::To(TimeRate { multiplier: -1.0 }),
            }),
            Request::SetTimeRate(SetTimeRateRequest {
                change: TimeRateChange::To(TimeRate { multiplier: f64::NAN }),
            }),
            Request::SetTime(SetTimeRequest {
                time: SimulationTime { seconds: f64::INFINITY },
            }),
            Request::StepTime(StepTimeRequest {
                duration: SimulationDuration { seconds: 0.0 },
            }),
            Request::SetTimeOverride(SetTimeOverrideRequest {
                rate: Some(TimeRate { multiplier: -1.0 }),
            }),
        ] {
            harness.send(&connection, request);
            let events = drain(&mut connection);
            assert!(is_denied(&events), "{events:?}");
        }

        assert_eq!(harness.server.world.time_flow(), TimeFlow::REAL_TIME);
    }

    #[test]
    fn requests_before_joining_are_denied() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(
            &connection,
            Request::SetTimeRate(SetTimeRateRequest {
                change: TimeRateChange::To(TimeRate { multiplier: 2.0 }),
            }),
        );
        assert!(is_denied(&drain(&mut connection)));
    }

    #[test]
    fn users_joining_and_leaving_are_announced_to_others() {
        let mut harness = Harness::new();
        let mut first = harness.connect();
        let second = harness.connect();
        let server_user = harness.connect();
        harness.send(&first, user("ada"));
        drain(&mut first);

        harness.send(&server_user, server_role());
        assert!(drain(&mut first).is_empty());

        harness.send(&second, user("grace"));
        let joined = drain(&mut first);
        let [Event::UserJoined(joined)] = joined.as_slice() else {
            panic!("{joined:?}");
        };
        assert_eq!(joined.user.name, "grace");

        let grace = joined.user.id;
        harness.send(&second, Request::Leave(LeaveRequest));
        assert_eq!(drain(&mut first), vec![Event::UserLeft(UserLeftEvent { user: grace })]);
    }
}
