mod dedicated_config;
mod dedicated_options;

pub use dedicated_config::DedicatedConfig;
pub use dedicated_options::DedicatedOptions;

use std::net::SocketAddr;

use anyhow::anyhow;
use tokio::sync::mpsc::unbounded_channel;
use xsa_commands::ipc::{IpcEndpoint, ProcessKind};
use xsa_commands::logging::{LoggingConfig, ShellOutputs};
use xsa_commands::repl::Repl;
use xsa_commands::script::Script;
use xsa_config::ConfigStore;
use xsa_proto::network::Connection;
use xsa_proto::network::{DisconnectReason, Identity, Listener};
use xsa_proto::session::SessionEnvironment;

use crate::operator::Operator;
use crate::{Peer, Server, World};

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
        let mut config = ConfigStore::<DedicatedConfig>::load(options.config.clone())?;

        if let Some(filter) = &options.log_filter_override {
            LoggingConfig::override_filter(&mut config, filter)?;
        }

        let shell_outputs = ShellOutputs::new(options.log_filter.clone(), &config.config().logging);
        let script = options.script.as_deref().map(Script::load).transpose()?;
        let world = World::load(&options.world)?;
        println!("simulation {}", world.simulation_id());
        let identity = Identity::load_or_generate(&options.identity)?;
        let address = SocketAddr::new(options.host, options.port);
        let listener = Listener::bind(&identity, address)?;
        println!("listening on {address}");
        println!("fingerprint {}", identity.fingerprint());

        let (peers, receiver) = unbounded_channel();
        let server = Server::spawn(world, receiver, SessionEnvironment::Dedicated)?;

        let operator_connection = Connection::local();
        peers
            .send(Peer::Operator(operator_connection.link))
            .map_err(|_| anyhow!("the simulation thread stopped"))?;
        let (invocation_sender, invocations) = unbounded_channel();
        let _endpoint = IpcEndpoint::spawn(
            options.name,
            ProcessKind::Server,
            invocation_sender.clone(),
            shell_outputs.ipc_transcript(),
        )?;

        let _repl = Repl::spawn(invocation_sender)?;
        let mut operator = tokio::spawn(Operator::run(
            operator_connection.connection,
            invocations,
            config,
            Box::new(move |config: &DedicatedConfig| shell_outputs.apply(&config.logging)),
            script,
        ));

        let stopped = loop {
            tokio::select! {
                pending = listener.accept() => {
                    let Some(pending) = pending else {
                        break Ok(());
                    };
                    let peers = peers.clone();

                    tokio::spawn(async move {
                        let remote = pending.remote_address();

                        match pending.establish().await {
                            Ok(link) => {
                                tracing::info!("{remote} connected");
                                let _ = peers.send(Peer::Client(link));
                            }
                            Err(err) => tracing::warn!("{remote} failed to connect: {err:#}"),
                        }
                    });
                }
                _ = tokio::signal::ctrl_c() => break Ok(()),
                result = &mut operator => break result.map_err(anyhow::Error::from).and_then(|stopped| stopped),
            }
        };

        tracing::info!("shutting down");
        tokio::task::spawn_blocking(move || server.stop()).await??;
        listener.close(DisconnectReason::ServerStopped).await;

        stopped
    }
}
