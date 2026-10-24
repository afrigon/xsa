use xsa_proto::request::Request;
use xsa_proto::session::SessionState;

use crate::router::Output;

pub trait ServerCommand: Send {
    fn request(&self) -> Request;

    fn describe(&self, state: Option<&SessionState>) -> Output;
}
