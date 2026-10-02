#[cfg(not(feature = "server"))]
compile_error!("xsa needs the `server` feature; `client` enables it too");

use std::net::IpAddr;
use std::path::PathBuf;

use usage::{Args, Cli, Subcommands};
use xsa_server::WorldOptions;
use xsa_server::dedicated::{self, DedicatedOptions};

#[derive(Cli)]
#[usage(
    bin = "xsa",
    version,
    about = "xsa space flight simulator",
    unknown_flags = "error",
    args_override_self = false
)]
#[cfg_attr(
    feature = "client",
    usage(default_subcommand = "client", default_subcommand_flags, default_subcommand_on_empty)
)]
struct Arguments {
    #[usage(subcommand)]
    command: Command,
}

#[derive(Subcommands)]
enum Command {
    #[cfg(feature = "client")]
    /// Run the game, with an integrated server unless --remote is given
    Client(ClientArguments),
    /// Run a dedicated server
    Server(ServerArguments),
}

#[derive(Args)]
struct WorldArguments {
    #[usage(long, default = "packs", help = "Directory containing the installed packs")]
    packs_directory: PathBuf,
    #[usage(
        long,
        delimiter = ',',
        default = "base",
        default = "system-solar",
        help = "Packs to load, in stack order (later packs override earlier ones)"
    )]
    packs: Vec<String>,
    #[usage(
        long,
        help = "Simulation to run, e.g. system-solar:sol; defaults to the only one installed"
    )]
    simulation: Option<String>,
}

#[cfg(feature = "client")]
#[derive(Args)]
struct ClientArguments {
    #[usage(
        long,
        env_fallback("USER", "USERNAME"),
        default = "player",
        help = "Player name shown to other players"
    )]
    player: String,
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
    #[usage(flatten)]
    world: WorldArguments,
}

#[derive(Args)]
struct ServerArguments {
    #[usage(long, default = "::1", help = "Address to listen on")]
    host: IpAddr,
    #[usage(long, default = "1969", help = "UDP port to listen on")]
    port: u16,
    #[usage(long, default = ".xsa/server", help = "Directory holding the server identity")]
    identity: PathBuf,
    #[usage(flatten)]
    world: WorldArguments,
}

impl WorldArguments {
    fn into_options(self) -> WorldOptions {
        WorldOptions {
            packs_directory: self.packs_directory,
            packs: self.packs,
            simulation: self.simulation,
        }
    }
}

fn main() -> anyhow::Result<()> {
    match Arguments::parse().command {
        #[cfg(feature = "client")]
        Command::Client(arguments) => run_client(arguments),
        Command::Server(arguments) => dedicated::run(DedicatedOptions {
            host: arguments.host,
            port: arguments.port,
            identity: arguments.identity,
            world: arguments.world.into_options(),
        }),
    }
}

#[cfg(feature = "client")]
fn run_client(arguments: ClientArguments) -> anyhow::Result<()> {
    use anyhow::Context;

    let remote = match arguments.remote {
        Some(address) => Some(xsa_client::RemoteServer {
            address,
            fingerprint: arguments.fingerprint.context("--remote requires --fingerprint")?,
        }),
        None => None,
    };
    xsa_client::run(xsa_client::ClientOptions {
        player: arguments.player,
        remote,
        world: arguments.world.into_options(),
    })
}
