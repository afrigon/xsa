mod app;
mod camera;
mod content;
mod input;
mod lighting;
mod mesh;
mod renderer;
mod vulkan;

use std::path::PathBuf;

use anyhow::Context;
use usage::Cli;
use winit::event_loop::EventLoop;
use xsa_proto::messages::ClientMessage;
use xsa_proto::network;
use xsa_server::{World, WorldOptions};

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
    #[usage(long, default = "packs", help = "Directory containing the installed packs")]
    packs_directory: PathBuf,
    #[usage(
        long,
        delimiter = ',',
        default = "base",
        default = "system-solar",
        help = "Packs for local play, in stack order (later packs override earlier ones)"
    )]
    packs: Vec<String>,
    #[usage(
        long,
        help = "Simulation for local play, e.g. system-solar:sol; defaults to the only one installed"
    )]
    simulation: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let connection = match arguments.remote {
        Some(address) => {
            let fingerprint = arguments.fingerprint.context("--remote requires --fingerprint")?;
            network::connect(&address, &fingerprint)?
        }
        None => xsa_server::start_local(World::load(&WorldOptions {
            packs_directory: arguments.packs_directory.clone(),
            packs: arguments.packs,
            simulation: arguments.simulation,
        })?)?,
    };
    connection.send(ClientMessage::Join)?;

    let event_loop = EventLoop::new()?;
    let mut app = app::App::new(connection, arguments.packs_directory);
    event_loop.run_app(&mut app)?;
    app.into_result()
}
