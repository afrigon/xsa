use tokio::sync::mpsc::unbounded_channel;
use winit::event_loop::EventLoop;
use xsa_commands::ipc::{IpcEndpoint, ProcessKind};
use xsa_commands::logging::{LoggingConfig, ShellOutputs};
use xsa_commands::repl::Repl;
use xsa_commands::script::Script;
use xsa_config::ConfigStore;
use xsa_proto::network::Connection;
use xsa_proto::request::{JoinRequest, Request, Role};
use xsa_proto::session::ServerSession;
use xsa_server::{Server, World};

use crate::ClientOptions;
use crate::app::App;
use crate::config::Config;

pub struct GameClient {
    options: ClientOptions,
}

impl GameClient {
    pub fn new(options: ClientOptions) -> GameClient {
        GameClient { options }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let packs_directory = self.options.world.packs_directory.clone();
        let config_path = match self.options.config {
            Some(path) => path,
            None => Config::default_path()?,
        };
        let mut config = ConfigStore::<Config>::load(config_path)?;

        if let Some(filter) = &self.options.log_filter_override {
            LoggingConfig::override_filter(&mut config, filter)?;
        }

        let shell_outputs = ShellOutputs::new(self.options.log_filter, &config.config().logging);
        let script = self.options.script.as_deref().map(Script::load).transpose()?;
        let connection = match self.options.remote {
            Some(remote) => Connection::remote(&remote.address, &remote.fingerprint)?,
            None => Server::start_local(World::load(&self.options.world)?)?,
        };
        let mut session = ServerSession::new(connection);
        session.send(Request::Join(JoinRequest {
            role: Role::Player {
                name: self.options.player,
            },
        }))?;

        let (invocation_sender, invocations) = unbounded_channel();
        let _endpoint = IpcEndpoint::spawn(
            self.options.name,
            ProcessKind::Client,
            invocation_sender.clone(),
            shell_outputs.ipc_transcript(),
        )?;

        if let Some(script) = script {
            script.spawn(invocation_sender.clone(), shell_outputs.script_transcript())?;
        }

        let _repl = Repl::spawn(invocation_sender)?;
        let event_loop = EventLoop::new()?;
        let mut app = App::new(
            session,
            invocations,
            shell_outputs,
            packs_directory,
            config,
            self.options.build,
        );
        event_loop.run_app(&mut app)?;

        app.into_result()
    }
}
