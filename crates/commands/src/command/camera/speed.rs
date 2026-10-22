use usage::Args;

use crate::command::{ClientCommand, Routable, Route};
use crate::value::Speed;

#[derive(Args)]
pub struct CameraSpeedCommand {
    #[usage(
        allow_negative_numbers,
        help = "Flying speed, e.g. 100m/s (m/s, km/s, Mm/s, Gm/s; a bare number is m/s)"
    )]
    pub speed: Speed,
}

impl Routable for CameraSpeedCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::CameraSpeed(self))
    }
}
