use usage::Args;

use crate::command::{ClientCommand, Routable, Route};
use crate::completion::{complete_body, complete_latitude, complete_longitude};
use crate::value::{Altitude, Latitude, Longitude};

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
        complete = complete_latitude,
        help = "Latitude in degrees, positive north, or where a body is overhead or underfoot: substellar, antistellar, near-side (the parent overhead) or far-side; defaults to the current one"
    )]
    pub latitude: Option<Latitude>,
    #[usage(
        long = "lon",
        allow_negative_numbers,
        complete = complete_longitude,
        help = "Longitude in degrees, positive east, a time of day at the latitude (dawn, sunrise, noon, sunset, dusk, midnight), near-side or far-side; defaults to the current one"
    )]
    pub longitude: Option<Longitude>,
    #[usage(
        long,
        complete = complete_body,
        help = "The star substellar, antistellar and times of day refer to, by id; defaults to the one lighting the body most"
    )]
    pub star: Option<String>,
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
