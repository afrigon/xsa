use xsa_proto::session::SessionState;

use crate::router::Output;

pub trait SessionCommand: Send {
    fn answer(&self, state: Option<&SessionState>) -> Output;
}
