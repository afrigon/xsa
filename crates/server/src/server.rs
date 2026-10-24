mod client;
mod handlers;
mod request_handler;

use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::sync::oneshot;
use xsa_proto::error::{ForbiddenCause, RequestError};
use xsa_proto::event::{Event, TickEvent, TimeChangedEvent, UserLeftEvent};
use xsa_proto::frame::ServerFrame;
use xsa_proto::network::{Connection, DisconnectReason};
use xsa_proto::reply::Reply;
use xsa_proto::request::Request;
use xsa_proto::session::SessionEnvironment;
use xsa_proto::user::{Actor, User};
use xsa_units::{SimulationDuration, TICK_RATE_HERTZ};

use crate::{LocalServer, Peer, ServerHandle, World};
use client::Client;
use request_handler::RequestHandler;

pub struct Server {
    world: World,
    environment: SessionEnvironment,
    peers: UnboundedReceiver<Peer>,
    stop: oneshot::Receiver<()>,
    clients: Vec<Client>,
    users: Vec<User>,
    next_user: u32,
}

impl Server {
    pub fn spawn(
        world: World,
        peers: UnboundedReceiver<Peer>,
        environment: SessionEnvironment,
    ) -> anyhow::Result<ServerHandle> {
        let (stop, stop_receiver) = oneshot::channel();
        let thread = thread::Builder::new()
            .name("simulation".into())
            .spawn(move || Server::new(world, peers, stop_receiver, environment).run())?;

        Ok(ServerHandle { stop, thread })
    }

    pub fn start_local(world: World) -> anyhow::Result<LocalServer> {
        let local = Connection::local();
        let (peers, receiver) = unbounded_channel();
        peers
            .send(Peer::Client(local.link))
            .expect("the receiver is alive until the server starts");
        let handle = Server::spawn(world, receiver, SessionEnvironment::Singleplayer)?;

        Ok(LocalServer {
            connection: local.connection,
            handle,
        })
    }

    fn new(
        world: World,
        peers: UnboundedReceiver<Peer>,
        stop: oneshot::Receiver<()>,
        environment: SessionEnvironment,
    ) -> Self {
        Self {
            world,
            environment,
            peers,
            stop,
            clients: Vec::new(),
            users: Vec::new(),
            next_user: 0,
        }
    }

    fn run(mut self) {
        let tick = Duration::from_secs_f64(1.0 / TICK_RATE_HERTZ);
        let mut next_tick = Instant::now();

        while self.stop.try_recv() == Err(oneshot::error::TryRecvError::Empty) {
            self.accept_peers();
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

        self.accept_peers();
        self.close_all(DisconnectReason::ServerStopped);
    }

    fn accept_peers(&mut self) {
        while let Ok(peer) = self.peers.try_recv() {
            self.admit(peer);
        }
    }

    fn close_all(&mut self, reason: DisconnectReason) {
        for client in &mut self.clients {
            client.close(reason.clone());
        }

        self.remove_disconnected();
    }

    fn admit(&mut self, peer: Peer) {
        match peer {
            Peer::Client(link) => self.clients.push(Client::new(link, None)),
            Peer::Operator(link) => {
                self.clients.push(Client::new(link, Some(Actor::Operator)));
                self.send_snapshot(self.clients.len() - 1);
            }
        }
    }

    fn handle_frames(&mut self) {
        for client in 0..self.clients.len() {
            while let Some(frame) = self.clients[client].receive() {
                let result = self.handle(client, frame.request);
                self.send(client, ServerFrame::Reply { id: frame.id, result });
            }
        }
    }

    fn handle(&mut self, client: usize, request: Request) -> Result<Reply, RequestError> {
        let needs_actor = !matches!(request, Request::Join(_));

        if needs_actor && self.clients[client].actor.is_none() {
            return Err(RequestError::Forbidden {
                cause: ForbiddenCause::NotJoined,
            });
        }

        request.check_session_environment(self.environment)?;

        match request {
            Request::Join(request) => request.handle(self, client).map(Reply::from),
            Request::SetTime(request) => request.handle(self, client).map(Reply::from),
            Request::StepTime(request) => request.handle(self, client).map(Reply::from),
            Request::SetTimeRate(request) => request.handle(self, client).map(Reply::from),
            Request::StepTimeRate(request) => request.handle(self, client).map(Reply::from),
            Request::SetTimeRateOverride(request) => request.handle(self, client).map(Reply::from),
        }
    }

    fn send_snapshot(&mut self, client: usize) {
        let state = self.world.state(self.users.clone(), self.environment);
        self.send(client, ServerFrame::Snapshot(state));
    }

    fn broadcast_time(&mut self) {
        self.broadcast(&Event::TimeChanged(TimeChangedEvent {
            time: self.world.time(),
            time_flow: self.world.time_flow(),
        }));
    }

    fn send(&mut self, client: usize, frame: ServerFrame) {
        self.clients[client].send(frame);
    }

    fn broadcast(&mut self, event: &Event) {
        for client in self.clients.iter_mut().filter(|client| client.actor.is_some()) {
            client.send(ServerFrame::Event(event.clone()));
        }
    }

    fn remove_disconnected(&mut self) {
        while let Some(index) = self.clients.iter().position(|client| client.disconnected.is_some()) {
            let client = self.clients.remove(index);

            if let Some(reason) = client.disconnected {
                self.disconnect(client.actor, &reason);
            }
        }
    }

    fn disconnect(&mut self, actor: Option<Actor>, reason: &DisconnectReason) {
        match actor {
            Some(Actor::User(user)) => {
                tracing::info!("user {} disconnected: {reason}", user.value);
                self.users.retain(|joined| joined.id != user);
                self.broadcast(&Event::UserLeft(UserLeftEvent { user }));
            }
            Some(Actor::Operator) => tracing::info!("the operator disconnected: {reason}"),
            None => tracing::info!("a client disconnected before joining: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use xsa_proto::event::UserJoinedEvent;
    use xsa_proto::frame::ClientFrame;
    use xsa_proto::frame::RequestId;
    use xsa_proto::reply::JoinReply;
    use xsa_proto::request::{
        JoinRequest, SetTimeRateOverrideRequest, SetTimeRateRequest, SetTimeRequest, StepTimeRateRequest,
        StepTimeRequest,
    };
    use xsa_proto::user::UserId;
    use xsa_units::{SimulationTime, TimeFlow, TimeRate, TimeRateStep};

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
            let world = world();
            let (_peers, receiver) = unbounded_channel();
            let (_stop, stop_receiver) = oneshot::channel();

            Self {
                server: Server::new(world, receiver, stop_receiver, environment),
                next_id: 0,
            }
        }

        fn connect(&mut self) -> Connection {
            let local = Connection::local();
            self.server.admit(Peer::Client(local.link));

            local.connection
        }

        fn connect_operator(&mut self) -> Connection {
            let local = Connection::local();
            self.server.admit(Peer::Operator(local.link));

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

    fn world() -> World {
        World::load(&WorldOptions {
            packs_directory: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")),
            packs: vec![
                "base".to_string(),
                "base-surfaces".to_string(),
                "system-solar".to_string(),
            ],
            simulation: None,
        })
        .unwrap()
    }

    fn drain(connection: &mut Connection) -> Vec<ServerFrame> {
        let mut frames = Vec::new();

        while let Ok(Some(frame)) = connection.poll() {
            frames.push(frame);
        }

        frames
    }

    fn join() -> Request {
        Request::Join(JoinRequest)
    }

    fn event(event: Event) -> ServerFrame {
        ServerFrame::Event(event)
    }

    fn accepted(id: RequestId) -> ServerFrame {
        ServerFrame::Reply {
            id,
            result: Ok(Reply::Empty),
        }
    }

    fn is_refused(frames: &[ServerFrame]) -> bool {
        matches!(frames, [ServerFrame::Reply { result: Err(_), .. }])
    }

    #[test]
    fn join_is_answered_with_a_snapshot_the_announcement_then_the_user() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        let id = harness.send(&connection, join());
        let frames = drain(&mut connection);
        let user = UserId { value: 0 };
        assert!(matches!(
            &frames[0],
            ServerFrame::Snapshot(state) if state.time_flow == TimeFlow::REAL_TIME && state.users.is_empty()
        ));
        assert_eq!(
            frames[1..],
            [
                event(Event::UserJoined(UserJoinedEvent {
                    user: User { id: user }
                })),
                ServerFrame::Reply {
                    id,
                    result: Ok(Reply::Join(JoinReply { user })),
                },
            ]
        );
    }

    #[test]
    fn the_operator_gets_a_snapshot_without_joining() {
        let mut harness = Harness::new();
        let mut operator = harness.connect_operator();
        assert!(matches!(drain(&mut operator).as_slice(), [ServerFrame::Snapshot(_)]));
    }

    #[test]
    fn joining_twice_is_a_conflict() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(&connection, join());
        drain(&mut connection);
        harness.send(&connection, join());
        assert!(matches!(
            drain(&mut connection).as_slice(),
            [ServerFrame::Reply {
                result: Err(RequestError::Conflict { .. }),
                ..
            }]
        ));
    }

    #[test]
    fn a_time_change_is_broadcast_before_the_reply() {
        let mut harness = Harness::new();
        let mut operator = harness.connect_operator();
        let mut observer = harness.connect();
        harness.send(&observer, join());
        drain(&mut operator);
        drain(&mut observer);

        let rate = TimeRate { multiplier: 100.0 };
        let id = harness.send(&operator, Request::SetTimeRate(SetTimeRateRequest { rate }));
        let changed = event(Event::TimeChanged(TimeChangedEvent {
            time: harness.server.world.time(),
            time_flow: TimeFlow {
                rate,
                time_override: None,
            },
        }));
        assert_eq!(drain(&mut operator), vec![changed.clone(), accepted(id)]);
        assert_eq!(drain(&mut observer), vec![changed]);
    }

    #[test]
    fn faster_and_slower_change_the_world_rate() {
        let mut harness = Harness::new();
        let connection = harness.connect_operator();

        for (step, multiplier) in [
            (TimeRateStep::Faster, 2.0),
            (TimeRateStep::Faster, 4.0),
            (TimeRateStep::Slower, 2.0),
            (TimeRateStep::Slower, 1.0),
            (TimeRateStep::Slower, 0.5),
            (TimeRateStep::Slower, 0.25),
            (TimeRateStep::Slower, 0.1),
            (TimeRateStep::Slower, 0.0),
        ] {
            harness.send(&connection, Request::StepTimeRate(StepTimeRateRequest { step }));
            assert_eq!(harness.server.world.time_flow().rate, TimeRate { multiplier });
        }
    }

    #[test]
    fn stepping_advances_time_by_the_step() {
        let mut harness = Harness::new();
        let connection = harness.connect_operator();
        let before = harness.server.world.time();
        let duration = SimulationDuration { seconds: 10.0 };
        harness.send(&connection, Request::StepTime(StepTimeRequest { duration }));
        assert_eq!(harness.server.world.time(), before.advanced_by(duration));
    }

    #[test]
    fn a_paused_world_does_not_advance() {
        let mut harness = Harness::new();
        let connection = harness.connect_operator();
        harness.send(
            &connection,
            Request::SetTimeRate(SetTimeRateRequest { rate: TimeRate::PAUSED }),
        );
        let before = harness.server.world.time();
        harness.server.world.advance(SimulationDuration { seconds: 1.0 });
        assert_eq!(harness.server.world.time(), before);
    }

    #[test]
    fn an_override_stops_time_and_keeps_the_rate() {
        let mut harness = Harness::new();
        let connection = harness.connect_operator();
        let rate = TimeRate { multiplier: 4.0 };
        harness.send(&connection, Request::SetTimeRate(SetTimeRateRequest { rate }));
        harness.send(
            &connection,
            Request::SetTimeRateOverride(SetTimeRateOverrideRequest {
                rate: Some(TimeRate::PAUSED),
            }),
        );
        let before = harness.server.world.time();
        harness.server.world.advance(SimulationDuration { seconds: 1.0 });
        assert_eq!(harness.server.world.time(), before);
        assert_eq!(harness.server.world.time_flow().rate, rate);

        harness.send(
            &connection,
            Request::SetTimeRateOverride(SetTimeRateOverrideRequest { rate: None }),
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
        let mut connection = harness.connect_operator();
        drain(&mut connection);

        harness.send(
            &connection,
            Request::SetTimeRateOverride(SetTimeRateOverrideRequest {
                rate: Some(TimeRate::PAUSED),
            }),
        );
        assert!(is_refused(&drain(&mut connection)));
        assert_eq!(harness.server.world.time_flow(), TimeFlow::REAL_TIME);
    }

    #[test]
    fn invalid_values_are_denied_without_changing_the_world() {
        let mut harness = Harness::new();
        let mut connection = harness.connect_operator();
        drain(&mut connection);

        for request in [
            Request::SetTimeRate(SetTimeRateRequest {
                rate: TimeRate { multiplier: -1.0 },
            }),
            Request::SetTimeRate(SetTimeRateRequest {
                rate: TimeRate { multiplier: f64::NAN },
            }),
            Request::SetTime(SetTimeRequest {
                time: SimulationTime { seconds: f64::INFINITY },
            }),
            Request::StepTime(StepTimeRequest {
                duration: SimulationDuration { seconds: 0.0 },
            }),
            Request::SetTimeRateOverride(SetTimeRateOverrideRequest {
                rate: Some(TimeRate { multiplier: -1.0 }),
            }),
        ] {
            harness.send(&connection, request);
            let frames = drain(&mut connection);
            assert!(is_refused(&frames), "{frames:?}");
        }

        assert_eq!(harness.server.world.time_flow(), TimeFlow::REAL_TIME);
    }

    #[test]
    fn requests_before_joining_are_forbidden() {
        let mut harness = Harness::new();
        let mut connection = harness.connect();
        harness.send(
            &connection,
            Request::SetTimeRate(SetTimeRateRequest {
                rate: TimeRate { multiplier: 2.0 },
            }),
        );
        assert!(matches!(
            drain(&mut connection).as_slice(),
            [ServerFrame::Reply {
                result: Err(RequestError::Forbidden {
                    cause: ForbiddenCause::NotJoined
                }),
                ..
            }]
        ));
    }

    #[test]
    fn users_joining_and_leaving_are_announced_to_everyone() {
        let mut harness = Harness::new();
        let mut first = harness.connect();
        let second = harness.connect();
        let mut operator = harness.connect_operator();
        harness.send(&first, join());
        drain(&mut first);
        drain(&mut operator);

        harness.send(&second, join());
        let grace = User {
            id: UserId { value: 1 },
        };
        let joined = vec![event(Event::UserJoined(UserJoinedEvent { user: grace.clone() }))];
        assert_eq!(drain(&mut first), joined);
        assert_eq!(drain(&mut operator), joined);

        drop(second);
        harness.server.handle_frames();
        harness.server.remove_disconnected();
        let left = vec![event(Event::UserLeft(UserLeftEvent { user: grace.id }))];
        assert_eq!(drain(&mut first), left);
        assert_eq!(drain(&mut operator), left);
    }

    #[test]
    fn stopping_closes_every_connection_with_server_stopped() {
        let LocalServer { mut connection, handle } = Server::start_local(world()).unwrap();
        handle.stop().unwrap();

        let reason = loop {
            match connection.poll() {
                Ok(_) => continue,
                Err(reason) => break reason,
            }
        };
        assert_eq!(reason, DisconnectReason::ServerStopped);
    }
}
