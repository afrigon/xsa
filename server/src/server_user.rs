use tokio::sync::mpsc::UnboundedReceiver;
use xsa_commands::router::{CommandExecutor, CommandInvocation, CommandRouter};
use xsa_proto::connection::Connection;
use xsa_proto::messages::{ClientMessage, Role};
use xsa_proto::session::ServerSession;

struct ServerUser {
    session: ServerSession,
    exit_requested: bool,
}

impl CommandExecutor for ServerUser {
    fn session(&mut self) -> &mut ServerSession {
        &mut self.session
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }
}

pub async fn run(connection: Connection, mut invocations: UnboundedReceiver<CommandInvocation>) -> anyhow::Result<()> {
    let mut user = ServerUser {
        session: ServerSession::new(connection),
        exit_requested: false,
    };
    user.session.send(ClientMessage::Join { role: Role::Server })?;
    let mut router = CommandRouter::default();
    let mut invocations_open = true;
    while !user.exit_requested {
        tokio::select! {
            event = user.session.receive() => router.handle_event(&event?, &user.session),
            invocation = invocations.recv(), if invocations_open && user.session.state().is_some() => match invocation {
                Some(invocation) => router.handle(invocation, &mut user),
                None => invocations_open = false,
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
    use tokio::sync::oneshot;
    use xsa_commands::router::{CommandExecution, Output};

    use super::*;
    use crate::{World, WorldOptions, start_local};

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
    async fn server_user_commands_reach_the_server_and_report_back() {
        let world = World::load(&WorldOptions {
            packs_directory: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../data")),
            packs: vec!["base".to_string(), "system-solar".to_string()],
            simulation: None,
        })
        .unwrap();
        let connection = start_local(world).unwrap();
        let (invocations, receiver) = unbounded_channel();
        let server_user = tokio::spawn(run(connection, receiver));

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
        server_user.await.unwrap().unwrap();
    }
}
