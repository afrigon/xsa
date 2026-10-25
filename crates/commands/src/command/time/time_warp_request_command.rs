use xsa_proto::request::{Request, SetTimeWarpRequest};
use xsa_proto::session::SessionState;
use xsa_units::TimeWarp;

use crate::command::ServerCommand;
use crate::router::Output;

// The warp `time warp` works out on the client, sent on as a request.
pub struct TimeWarpRequestCommand {
    pub warp: TimeWarp,
    // The reply once the server accepts it.
    pub description: String,
}

impl ServerCommand for TimeWarpRequestCommand {
    fn request(&self) -> Request {
        Request::SetTimeWarp(SetTimeWarpRequest { warp: Some(self.warp) })
    }

    fn describe(&self, _state: Option<&SessionState>) -> Output {
        Output::success(self.description.clone())
    }
}
