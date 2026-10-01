mod app;
mod camera;
mod input;
mod mesh;
mod renderer;
mod vulkan;

use anyhow::Context;
use usage::Cli;
use winit::event_loop::EventLoop;
use xsa_core::simulation::Simulation;
use xsa_proto::messages::ClientMessage;
use xsa_proto::network;

#[derive(Cli)]
#[usage(
    bin = "xsa",
    version,
    about = "xsa space flight simulator",
    unknown_flags = "error",
    args_override_self = false
)]
struct Arguments {
    #[usage(
        long,
        requires("--fingerprint"),
        help = "Connect to a remote server at <host>:<port>; IPv6 hosts go in brackets, e.g. [::1]:1969"
    )]
    remote: Option<String>,
    #[usage(
        long,
        requires("--remote"),
        help = "Certificate fingerprint the remote server printed at startup"
    )]
    fingerprint: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let connection = match arguments.remote {
        Some(address) => {
            let fingerprint = arguments.fingerprint.context("--remote requires --fingerprint")?;
            network::connect(&address, &fingerprint)?
        }
        None => xsa_server::start_local(Simulation::solar_system())?,
    };
    connection.send(ClientMessage::Join)?;

    let event_loop = EventLoop::new()?;
    let mut app = app::App::new(connection);
    event_loop.run_app(&mut app)?;
    app.into_result()
}
