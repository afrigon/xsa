#[cfg(feature = "client")]
use super::ClientCommand;
use super::{LocalCommand, ScriptCommand, ServerCommand, SessionCommand, WaitCommand};

pub enum Route {
    #[cfg(feature = "client")]
    Client(ClientCommand),
    // Handled by the process that received the command, whichever kind it is; it never reaches the server.
    Local(LocalCommand),
    Server(Box<dyn ServerCommand>),
    Session(Box<dyn SessionCommand>),
    // Run by the router of the process that received the command, before joining too.
    Script(ScriptCommand),
    Wait(WaitCommand),
    Exit,
}
