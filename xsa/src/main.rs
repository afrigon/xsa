#[cfg(not(feature = "server"))]
compile_error!("xsa needs the `server` feature; `client` enables it too");

mod arguments;
#[cfg(feature = "client")]
mod client_arguments;
mod command;
mod completion_arguments;
mod ipc_arguments;
mod server_arguments;
mod world_arguments;

use arguments::Arguments;

fn main() -> anyhow::Result<()> {
    Arguments::parse().command.run()
}
