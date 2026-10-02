use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::thread;

use tokio::sync::mpsc::unbounded_channel;
use xsa_proto::network::{self, Identity};

use crate::{Server, World, WorldOptions};

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
        }
    }
    Ok(())
}
