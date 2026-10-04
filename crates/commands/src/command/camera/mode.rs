use usage::Args;

use super::CameraMode;
use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct CameraModeCommand {
    #[usage(value_enum)]
    pub mode: CameraMode,
}

impl Routable for CameraModeCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::CameraMode(self))
    }
}
