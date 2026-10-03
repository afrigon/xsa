use tokio::sync::mpsc::UnboundedReceiver;
use xsa_commands::dispatcher::{CommandHost, Dispatcher, Invocation};
use xsa_proto::connection::Connection;
use xsa_proto::messages::{ClientMessage, Role};
use xsa_proto::session::ServerSession;

struct ConsoleHost {
    session: ServerSession,
    exit_requested: bool,
}

impl CommandHost for ConsoleHost {
    fn session(&mut self) -> &mut ServerSession {
        &mut self.session
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }
}

pub async fn run(connection: Connection, mut invocations: UnboundedReceiver<Invocation>) -> anyhow::Result<()> {
    let mut host = ConsoleHost {
        session: ServerSession::new(connection),
        exit_requested: false,
    };
    host.session.send(ClientMessage::Join { role: Role::Console })?;
    let mut dispatcher = Dispatcher::default();
    let mut invocations_open = true;
    while !host.exit_requested {
        tokio::select! {
            event = host.session.receive() => dispatcher.handle_event(&event?, &host.session),
            invocation = invocations.recv(), if invocations_open && host.session.state().is_some() => match invocation {
                Some(invocation) => dispatcher.dispatch(invocation, &mut host),
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
    use xsa_commands::dispatcher::Output;

    use super::*;
    use crate::{World, WorldOptions, start_local};

    async fn invoke(invocations: &UnboundedSender<Invocation>, line: &str) -> Output {
        let (reply, receiver) = oneshot::channel();
        let words = xsa_commands::words::split(line).unwrap();
        invocations.send(Invocation { words, reply }).unwrap();
        receiver.await.unwrap()
    }

    #[tokio::test]
    async fn console_commands_reach_the_server_and_report_back() {
        let world = World::load(&WorldOptions {
            packs_directory: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../packs")),
            packs: vec!["base".to_string(), "system-solar".to_string()],
            simulation: None,
        })
        .unwrap();
        let connection = start_local(world).unwrap();
        let (invocations, receiver) = unbounded_channel();
        let console = tokio::spawn(run(connection, receiver));

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
        console.await.unwrap().unwrap();
    }
}
