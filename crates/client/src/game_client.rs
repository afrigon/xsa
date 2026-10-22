use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::watch;
use winit::event_loop::EventLoop;
use xsa_commands::ipc::{IpcEndpoint, ProcessKind};
use xsa_commands::repl::Repl;
use xsa_proto::connection::Connection;
use xsa_proto::message::{ClientMessage, Join, Role};
use xsa_proto::session::ServerSession;
use xsa_server::{Server, World};

use crate::ClientOptions;
use crate::app::App;
use crate::document::ConfigDocument;

pub struct GameClient {
    options: ClientOptions,
}

impl GameClient {
    pub fn new(options: ClientOptions) -> GameClient {
        GameClient { options }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let packs_directory = self.options.world.packs_directory.clone();
        let config = ConfigDocument::load(ConfigDocument::default_path()?)?;
        let connection = match self.options.remote {
            Some(remote) => Connection::remote(&remote.address, &remote.fingerprint)?,
            None => Server::start_local(World::load(&self.options.world)?)?,
        };
        let mut session = ServerSession::new(connection);
        session.send(ClientMessage::Join(Join {
            role: Role::Player {
                name: self.options.player,
            },
        }))?;

        let (invocation_sender, invocations) = unbounded_channel();
        let (ipc_logging, ipc_logging_receiver) = watch::channel(config.config().logging.ipc);
        let _endpoint = IpcEndpoint::spawn(
            self.options.name,
            ProcessKind::Client,
            invocation_sender.clone(),
            ipc_logging_receiver,
        )?;
        let _repl = Repl::spawn(invocation_sender)?;
        let event_loop = EventLoop::new()?;
        let mut app = App::new(session, invocations, ipc_logging, packs_directory, config);
        event_loop.run_app(&mut app)?;

        app.into_result()
    }
}
