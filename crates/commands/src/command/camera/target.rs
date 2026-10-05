use usage::Args;

use crate::command::{Availability, ClientCommand, Routable, Route};
use xsa_core::simulation::BodyCategory;

use crate::completion::{complete_body_category, complete_target};
use crate::value::{Distance, Target};

#[derive(Args)]
pub struct CameraTargetCommand {
    #[usage(complete = complete_target)]
    pub target: Target,
    #[usage(
        complete = complete_body_category,
        help = "With next or previous, only stop at bodies of this category, e.g. moon"
    )]
    pub category: Option<BodyCategory>,
    #[usage(
        long,
        help = "Distance from the body's center, e.g. 200km (m, km, Mm, Gm; a bare number is km)"
    )]
    pub distance: Option<Distance>,
    #[usage(long, allow_negative_numbers, help = "Pitch in degrees; positive looks up")]
    pub pitch: Option<f64>,
    #[usage(long, allow_negative_numbers, help = "Yaw in degrees around the ecliptic north pole")]
    pub yaw: Option<f64>,
}

impl Routable for CameraTargetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::CameraTarget(self))
    }

    fn availability(&self) -> Availability {
        Availability::InGame
    }
}
