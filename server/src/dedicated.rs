mod dedicated_options;

pub use dedicated_options::DedicatedOptions;

use std::net::SocketAddr;
use std::thread;

use anyhow::anyhow;
use tokio::sync::mpsc::unbounded_channel;
use xsa_commands::ipc::{InstanceKind, IpcEndpoint};
use xsa_commands::repl::Repl;
use xsa_proto::connection::Connection;
use xsa_proto::network::{Identity, Listener};

use crate::server_user::ServerUser;
use crate::{Server, World};

pub struct DedicatedServer {
    options: DedicatedOptions,
}

impl DedicatedServer {
    pub fn new(options: DedicatedOptions) -> DedicatedServer {
        DedicatedServer { options }
    }

    pub fn run(self) -> anyhow::Result<()> {
        tokio::runtime::Runtime::new()?.block_on(self.serve())
    }

    async fn serve(self) -> anyhow::Result<()> {
        let options = self.options;
        let world = World::load(&options.world)?;
        println!("simulation {}", world.simulation_id());
        let identity = Identity::load_or_generate(&options.identity)?;
        let address = SocketAddr::new(options.host, options.port);
        let listener = Listener::bind(&identity, address)?;
        println!("listening on {address}");
        println!("fingerprint {}", identity.fingerprint());

        let (new_clients, receiver) = unbounded_channel();
        thread::Builder::new()
            .name("simulation".into())
            .spawn(move || Server::new(world, receiver).run())?;

        let server_user_connection = Connection::local();
        new_clients
            .send(server_user_connection.link)
            .map_err(|_| anyhow!("the simulation thread stopped"))?;
        let (invocation_sender, invocations) = unbounded_channel();
        let _endpoint = IpcEndpoint::spawn(options.instance, InstanceKind::Server, invocation_sender.clone())?;
        let _repl = Repl::spawn(invocation_sender)?;
        let mut server_user = tokio::spawn(ServerUser::run(server_user_connection.connection, invocations));

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
                                tracing::info!("{remote} connected");
                                let _ = new_clients.send(link);
                            }
                            Err(err) => tracing::warn!("{remote} failed to connect: {err:#}"),
                        }
                    });
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("shutting down");
                    break;
                }
                result = &mut server_user => {
                    result??;
                    tracing::info!("shutting down");
                    break;
                }
            }
        }

        Ok(())
    }
}
