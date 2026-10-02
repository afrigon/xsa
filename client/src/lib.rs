mod app;
mod camera;
mod content;
mod input;
mod lighting;
mod mesh;
mod renderer;
mod vulkan;

use winit::event_loop::EventLoop;
use xsa_proto::messages::ClientMessage;
use xsa_proto::network;
use xsa_server::{World, WorldOptions};

pub struct RemoteServer {
    pub address: String,
    pub fingerprint: String,
}

pub struct ClientOptions {
    pub remote: Option<RemoteServer>,
    pub world: WorldOptions,
}

pub fn run(options: ClientOptions) -> anyhow::Result<()> {
    let packs_directory = options.world.packs_directory.clone();
    let connection = match options.remote {
        Some(remote) => network::connect(&remote.address, &remote.fingerprint)?,
        None => xsa_server::start_local(World::load(&options.world)?)?,
    };
    connection.send(ClientMessage::Join)?;

    let event_loop = EventLoop::new()?;
    let mut app = app::App::new(connection, packs_directory);
    event_loop.run_app(&mut app)?;
    app.into_result()
}
