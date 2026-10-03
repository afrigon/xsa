#[cfg(not(feature = "server"))]
compile_error!("xsa needs the `server` feature; `client` enables it too");

mod arguments;
#[cfg(feature = "client")]
mod client_arguments;
mod command;
mod completion_arguments;
mod ipc_arguments;
mod logging;
mod server_arguments;
mod world_arguments;

use arguments::Arguments;
use logging::Logging;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    Logging::from_verbosity(arguments.verbose, arguments.quiet).install();

    arguments.command.run()
}
