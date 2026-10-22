#[cfg(not(feature = "server"))]
compile_error!("xsa needs the `server` feature; `client` enables it too");

mod arguments;
#[cfg(feature = "client")]
mod client_arguments;
mod command;
mod completion_arguments;
#[cfg(feature = "client")]
mod current_build;
mod ipc_arguments;
mod server_arguments;
mod world_arguments;

use arguments::Arguments;
use xsa_commands::console_log::ConsoleLog;
use xsa_logging::Logging;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    Logging::new(arguments.verbosity.verbosity()).install(ConsoleLog::writer);

    arguments.command.run()
}
