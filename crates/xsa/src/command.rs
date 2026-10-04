use usage::Subcommands;

#[cfg(feature = "client")]
use crate::client_arguments::ClientArguments;
use crate::completion_arguments::CompletionArguments;
use crate::ipc_arguments::IpcArguments;
use crate::server_arguments::ServerArguments;

#[derive(Subcommands)]
pub enum Command {
    #[cfg(feature = "client")]
    /// Run the game, with an integrated server unless --remote is given
    Client(ClientArguments),
    /// Run a dedicated server
    Server(ServerArguments),
    /// Send a command to a running xsa instance, e.g. xsa ipc time rate 100
    Ipc(IpcArguments),
    /// Print the shell completion script, e.g. xsa completion fish | source
    Completion(CompletionArguments),
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "client")]
            Command::Client(arguments) => arguments.run(),
            Command::Server(arguments) => arguments.run(),
            Command::Ipc(arguments) => arguments.run(),
            Command::Completion(arguments) => arguments.run(),
        }
    }
}
