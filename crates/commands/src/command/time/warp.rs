use usage::Args;
use xsa_core::daylight::TimeOfDay;
use xsa_proto::request::{Request, SetTimeWarpRequest};
use xsa_proto::session::SessionState;

use super::TimeCommand;
use crate::command::{ClientCommand, Routable, Route, ServerCommand};
use crate::completion::{complete_body, complete_time_of_day};
use crate::router::Output;

#[derive(Args)]
#[usage(group("destination", required))]
pub struct TimeWarpCommand {
    #[usage(
        complete = complete_time_of_day,
        group = "destination",
        help = "The next time of day under the camera: dawn, sunrise, noon, sunset, dusk or midnight"
    )]
    pub time_of_day: Option<TimeOfDay>,
    #[usage(long, group = "destination", help = "Stop the running warp where it is")]
    pub cancel: bool,
    #[usage(
        long,
        complete = complete_body,
        conflicts = "--cancel",
        help = "The star the time of day refers to, by id; defaults to the one lighting the body most"
    )]
    pub star: Option<String>,
    #[usage(
        long,
        conflicts = "--cancel",
        help = "Jump there at once instead of easing; animation.time off does the same"
    )]
    pub instant: bool,
}

impl Routable for TimeWarpCommand {
    fn route(self) -> Route {
        if self.cancel {
            return Route::Server(Box::new(TimeWarpCancelCommand));
        }

        Route::Client(ClientCommand::TimeWarp(self))
    }
}

struct TimeWarpCancelCommand;

impl ServerCommand for TimeWarpCancelCommand {
    fn request(&self) -> Request {
        Request::SetTimeWarp(SetTimeWarpRequest { warp: None })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
