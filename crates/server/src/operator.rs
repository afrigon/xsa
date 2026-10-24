use tokio::sync::mpsc::UnboundedReceiver;
use xsa_commands::command::LocalCommand;
use xsa_commands::completion::{CompletionCandidate, CompletionKind};
use xsa_commands::router::{CommandExecutor, CommandInvocation, CommandRouter, Output};
use xsa_config::ConfigStore;
use xsa_proto::network::Connection;
use xsa_proto::session::ServerSession;

use crate::DedicatedConfig;

pub(crate) type ApplyConfig = Box<dyn Fn(&DedicatedConfig) + Send>;

pub(crate) struct Operator {
    session: ServerSession,
    exit_requested: bool,
    config: ConfigStore<DedicatedConfig>,
    apply_config: ApplyConfig,
}

impl CommandExecutor for Operator {
    fn session(&mut self) -> &mut ServerSession {
        &mut self.session
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }

    fn completion_values(&self, kind: CompletionKind) -> Vec<CompletionCandidate> {
        CompletionCandidate::config(kind, self.config.keys())
    }

    fn run_local(&mut self, command: LocalCommand) -> Output {
        let result = command.run(&mut self.config);
        (self.apply_config)(self.config.config());

        match result {
            Ok(text) => Output::success(text),
            Err(err) => Output::failure(format!("{err:#}")),
        }
    }
}

impl Operator {
    pub async fn run(
        connection: Connection,
        mut invocations: UnboundedReceiver<CommandInvocation>,
        config: ConfigStore<DedicatedConfig>,
        apply_config: ApplyConfig,
    ) -> anyhow::Result<()> {
        let mut operator = Operator {
            session: ServerSession::operator(connection),
            exit_requested: false,
            config,
            apply_config,
        };
        let mut router = CommandRouter::default();
        let mut invocations_open = true;

        while !operator.exit_requested {
            tokio::select! {
                frame = operator.session.receive() => {
                    router.handle_frame(&frame?, &operator.session);
                    router.resume(&mut operator);
                }
                invocation = invocations.recv(), if invocations_open => match invocation {
                    Some(invocation) => router.handle(invocation, &mut operator),
                    None => invocations_open = false,
                },
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
    use tokio::sync::oneshot;
    use xsa_commands::router::{CommandExecution, Output};

    use super::*;
    use xsa_proto::network::Connection;
    use xsa_proto::session::SessionEnvironment;

    use crate::{Peer, Server, World, WorldOptions};

    async fn invoke(invocations: &UnboundedSender<CommandInvocation>, line: &str) -> Output {
        let (reply, receiver) = oneshot::channel();
        let words = xsa_commands::words::split(line).unwrap();
        invocations
            .send(CommandInvocation::Execute(CommandExecution {
                words,
                reply,
                styled: false,
            }))
            .unwrap();
        receiver.await.unwrap()
    }

    #[tokio::test]
    async fn operator_commands_reach_the_server_and_report_back() {
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
        let local = Connection::local();
        let (peers, peer_receiver) = unbounded_channel();
        peers.send(Peer::Operator(local.link)).unwrap();
        let server = Server::spawn(world, peer_receiver, SessionEnvironment::Dedicated).unwrap();
        let (invocations, receiver) = unbounded_channel();
        let config_path = std::env::temp_dir().join(format!("xsa-server-operator-{}.kdl", std::process::id()));
        let config = ConfigStore::<DedicatedConfig>::load(config_path).unwrap();
        let operator = tokio::spawn(Operator::run(local.connection, receiver, config, Box::new(|_| {})));

        let rate = invoke(&invocations, "time rate 5").await;
        assert!(rate.succeeded && rate.text.contains("rate 5×"), "{rate:?}");
        let paused = invoke(&invocations, "time pause").await;
        assert!(paused.succeeded && paused.text.ends_with("paused"), "{paused:?}");
        let step = invoke(&invocations, "time step 60t").await;
        assert!(step.succeeded, "{step:?}");
        let denied = invoke(&invocations, "time rate -1").await;
        assert!(!denied.succeeded && denied.text.contains("at least 0"), "{denied:?}");
        let camera = invoke(&invocations, "camera mode debug").await;
        assert!(!camera.succeeded, "{camera:?}");

        invoke(&invocations, "exit").await;
        operator.await.unwrap().unwrap();
        server.stop().unwrap();
    }
}
