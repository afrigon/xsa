use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::thread;

use anyhow::anyhow;
use tokio::sync::mpsc::unbounded_channel;
use xsa_proto::connection::Connection;
use xsa_proto::network::{self, Identity};

use crate::{Server, World, WorldOptions, console};

pub struct DedicatedOptions {
    pub host: IpAddr,
    pub port: u16,
    pub identity: PathBuf,
    pub world: WorldOptions,
}

#[tokio::main]
pub async fn run(options: DedicatedOptions) -> anyhow::Result<()> {
    let world = World::load(&options.world)?;
    println!("simulation {}", world.simulation().id());
    let identity = Identity::load_or_generate(&options.identity)?;
    let address = SocketAddr::new(options.host, options.port);
    let listener = network::listen(&identity, address)?;
    println!("listening on {address}");
    println!("fingerprint {}", identity.fingerprint());

    let (new_clients, receiver) = unbounded_channel();
    thread::Builder::new()
        .name("simulation".into())
        .spawn(move || Server::new(world, receiver).run())?;

    let (console_connection, console_link) = Connection::local();
    new_clients
        .send(console_link)
        .map_err(|_| anyhow!("the simulation thread stopped"))?;
    let (_invocation_sender, invocations) = unbounded_channel();
    let mut console = tokio::spawn(console::run(console_connection, invocations));

    loop {
        tokio::select! {
            pending = listener.accept() => {
                let Some(pending) = pending else {
                    break;
                };
                let new_clients = new_clients.clone();
                tokio::spawn(async move {
                    let remote = pending.remote_address();
                    match pending.establish().await {
                        Ok(link) => {
                            println!("{remote} connected");
                            let _ = new_clients.send(link);
                        }
                        Err(err) => eprintln!("{remote} failed to connect: {err:#}"),
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => {
                println!("shutting down");
                break;
            }
            result = &mut console => {
                result??;
                println!("shutting down");
                break;
            }
        }
    }
    Ok(())
}
