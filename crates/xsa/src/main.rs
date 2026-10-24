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
use xsa_commands::shell::ShellLog;
use xsa_logging::Logging;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let log_filter_override = arguments.verbosity.filter_override();
    let log_filter = Logging::new(log_filter_override.clone().unwrap_or_default()).install(ShellLog::writer);

    arguments.command.run(log_filter, log_filter_override)
}
