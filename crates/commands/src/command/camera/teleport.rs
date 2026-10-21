use usage::Args;

use crate::command::{ClientCommand, Routable, Route};
use crate::completion::complete_body;
use crate::value::Altitude;

#[derive(Args)]
pub struct CameraTeleportCommand {
    #[usage(
        complete = complete_body,
        help = "The body to stand on, by id; defaults to the one the camera is around"
    )]
    pub body: Option<String>,
    #[usage(
        long = "lat",
        allow_negative_numbers,
        help = "Latitude in degrees, positive north; defaults to the current one"
    )]
    pub latitude: Option<f64>,
    #[usage(
        long = "lon",
        allow_negative_numbers,
        help = "Longitude in degrees, positive east; defaults to the current one"
    )]
    pub longitude: Option<f64>,
    #[usage(
        long,
        allow_negative_numbers,
        help = "Feet above the ground, e.g. 100m (m, km, Mm, Gm; a bare number is km), the eyes at eye height above them; defaults to the current one"
    )]
    pub altitude: Option<Altitude>,
    #[usage(
        long,
        allow_negative_numbers,
        help = "Heading in degrees clockwise from north; defaults to the current one"
    )]
    pub yaw: Option<f64>,
    #[usage(
        long,
        allow_negative_numbers,
        help = "Pitch in degrees above the horizon; defaults to the current one"
    )]
    pub pitch: Option<f64>,
}

impl Routable for CameraTeleportCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::CameraTeleport(self))
    }
}
