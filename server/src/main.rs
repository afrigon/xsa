use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::thread;

use tokio::sync::mpsc::unbounded_channel;
use usage::Cli;
use xsa_core::simulation::Simulation;
use xsa_proto::network::{self, Identity};
use xsa_server::Server;

#[derive(Cli)]
#[usage(
    bin = "xsa-server",
    version,
    about = "Dedicated xsa server",
    unknown_flags = "error",
    args_override_self = false
)]
struct Arguments {
    #[usage(long, default = "::1", help = "Address to listen on")]
    host: IpAddr,
    #[usage(long, default = "1969", help = "UDP port to listen on")]
    port: u16,
    #[usage(long, default = ".xsa/server", help = "Directory holding the server identity")]
    identity: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let identity = Identity::load_or_generate(&arguments.identity)?;
    let address = SocketAddr::new(arguments.host, arguments.port);
    let listener = network::listen(&identity, address)?;
    println!("listening on {address}");
    println!("fingerprint {}", identity.fingerprint());

    let (new_clients, receiver) = unbounded_channel();
    thread::Builder::new()
        .name("simulation".into())
        .spawn(move || Server::new(Simulation::solar_system(), receiver).run())?;

    loop {
        tokio::select! {
            pending = listener.accept() => {
                let Some(pending) = pending else {
                    break;
                };
                let new_clients = new_clients.clone();
                tokio::spawn(async move {
                    let remote = pending.remote_address();
                    match pending.establish().await {
                        Ok(link) => {
                            println!("{remote} connected");
                            let _ = new_clients.send(link);
                        }
                        Err(err) => eprintln!("{remote} failed to connect: {err:#}"),
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => {
                println!("shutting down");
                break;
            }
        }
    }
    Ok(())
}
