mod cli;
mod launchpad_context;
mod shell;

use std::io;
use std::process::ExitCode;

use cli::{Arguments, CommandHandler};
use launchpad_context::LaunchpadContext;
use shell::Shell;
use xsa_logging::Logging;

#[tokio::main]
async fn main() -> ExitCode {
    let arguments = Arguments::parse();
    let verbosity = arguments.verbosity.verbosity();
    Logging::new(arguments.verbosity.filter_override().unwrap_or_default()).install(io::stderr);
    let mut context = LaunchpadContext::new(Shell::new(verbosity));

    match arguments.command.handle(&mut context).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = context.shell().error(&error);

            ExitCode::FAILURE
        }
    }
}
