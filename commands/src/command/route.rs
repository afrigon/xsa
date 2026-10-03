#[cfg(feature = "client")]
use super::ClientCommand;
use super::{ServerCommand, SessionCommand};

pub enum Route {
    #[cfg(feature = "client")]
    Client(ClientCommand),
    Server(Box<dyn ServerCommand>),
    Session(Box<dyn SessionCommand>),
    Exit,
}
