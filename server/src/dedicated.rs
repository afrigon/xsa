use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::thread;

use anyhow::anyhow;
use tokio::sync::mpsc::unbounded_channel;
use xsa_commands::ipc::{self, InstanceKind};
use xsa_commands::repl;
use xsa_proto::connection::Connection;
use xsa_proto::network::{self, Identity};

use crate::{Server, World, WorldOptions, server_user};

pub struct DedicatedOptions {
    pub instance: Option<String>,
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

    let (server_user_connection, server_user_link) = Connection::local();
    new_clients
        .send(server_user_link)
        .map_err(|_| anyhow!("the simulation thread stopped"))?;
    let (invocation_sender, invocations) = unbounded_channel();
    let _endpoint = ipc::spawn(options.instance, InstanceKind::Server, invocation_sender.clone())?;
    let _repl = repl::spawn(invocation_sender)?;
    let mut server_user = tokio::spawn(server_user::run(server_user_connection, invocations));

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
            result = &mut server_user => {
                result??;
                println!("shutting down");
                break;
            }
        }
    }
    Ok(())
}
