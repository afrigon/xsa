mod app;
mod camera;
mod content;
mod input;
mod lighting;
mod mesh;
mod renderer;
mod vulkan;

use tokio::sync::mpsc::unbounded_channel;
use winit::event_loop::EventLoop;
use xsa_commands::ipc::{InstanceKind, IpcEndpoint};
use xsa_commands::repl::Repl;
use xsa_proto::connection::Connection;
use xsa_proto::message::{ClientMessage, Join, Role};
use xsa_proto::session::ServerSession;
use xsa_server::{Server, World, WorldOptions};

pub struct RemoteServer {
    pub address: String,
    pub fingerprint: String,
}

pub struct ClientOptions {
    pub instance: Option<String>,
    pub player: String,
    pub remote: Option<RemoteServer>,
    pub world: WorldOptions,
}

pub fn run(options: ClientOptions) -> anyhow::Result<()> {
    let packs_directory = options.world.packs_directory.clone();
    let connection = match options.remote {
        Some(remote) => Connection::remote(&remote.address, &remote.fingerprint)?,
        None => Server::start_local(World::load(&options.world)?)?,
    };
    let mut session = ServerSession::new(connection);
    session.send(ClientMessage::Join(Join {
        role: Role::Player { name: options.player },
    }))?;

    let (invocation_sender, invocations) = unbounded_channel();
    let _endpoint = IpcEndpoint::spawn(options.instance, InstanceKind::Client, invocation_sender.clone())?;
    let _repl = Repl::spawn(invocation_sender)?;
    let event_loop = EventLoop::new()?;
    let mut app = app::App::new(session, invocations, packs_directory);
    event_loop.run_app(&mut app)?;
    app.into_result()
}
