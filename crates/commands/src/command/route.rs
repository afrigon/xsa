#[cfg(feature = "client")]
use super::ClientCommand;
use super::{LocalCommand, ServerCommand, SessionCommand};

pub enum Route {
    #[cfg(feature = "client")]
    Client(ClientCommand),
    // Handled by the process that received the command, whichever kind it is; it never reaches the server.
    Local(LocalCommand),
    Server(Box<dyn ServerCommand>),
    Session(Box<dyn SessionCommand>),
    Exit,
}
