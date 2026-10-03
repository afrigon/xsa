use xsa_proto::message::ClientMessage;
use xsa_proto::session::SessionState;

use crate::router::Output;

pub trait ServerCommand: Send {
    fn message(&self) -> ClientMessage;

    fn describe(&self, state: Option<&SessionState>) -> Output;
}
