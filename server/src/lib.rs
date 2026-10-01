use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use xsa_core::simulation::Simulation;
use xsa_proto::connection::{ClientLink, Connection};
use xsa_proto::messages::{BodyDefinition, ClientMessage, ServerEvent};

const TICK_RATE_HERTZ: f64 = 20.0;

pub struct Server {
    simulation: Simulation,
    new_clients: UnboundedReceiver<ClientLink>,
    clients: Vec<Client>,
    time: f64,
}

struct Client {
    link: ClientLink,
    joined: bool,
}

impl Server {
    pub fn new(simulation: Simulation, new_clients: UnboundedReceiver<ClientLink>) -> Self {
        Self {
            simulation,
            new_clients,
            clients: Vec::new(),
            time: 0.0,
        }
    }

    pub fn run(mut self) {
        let tick = Duration::from_secs_f64(1.0 / TICK_RATE_HERTZ);
        let mut next_tick = Instant::now();
        while self.accept_clients() {
            self.handle_messages();
            self.time += tick.as_secs_f64();
            self.broadcast(&ServerEvent::Tick { time: self.time });

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
        let simulation = &self.simulation;
        self.clients.retain_mut(|client| {
            loop {
                match client.link.messages.try_recv() {
                    Ok(ClientMessage::Join) => {
                        client.joined = true;
                        let welcome = ServerEvent::Welcome {
                            bodies: body_definitions(simulation),
                        };
                        if client.link.events.send(welcome).is_err() {
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

pub fn start_local(simulation: Simulation) -> anyhow::Result<Connection> {
    let (connection, link) = Connection::local();
    let (new_clients, receiver) = unbounded_channel();
    new_clients
        .send(link)
        .expect("the receiver is alive until the server starts");
    thread::Builder::new()
        .name("integrated server".into())
        .spawn(move || Server::new(simulation, receiver).run())?;
    Ok(connection)
}

fn body_definitions(simulation: &Simulation) -> Vec<BodyDefinition> {
    simulation
        .bodies
        .iter()
        .map(|body| BodyDefinition {
            id: body.id.clone(),
            position: body.position.to_array(),
            orientation: body.orientation.to_array(),
            radius: body.radius,
        })
        .collect()
}
