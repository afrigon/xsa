use std::path::PathBuf;

use usage::Args;

use crate::command::{ClientCommand, Routable, Route};
use crate::value::{Duration, Region};

#[derive(Args)]
pub struct CameraSnapCommand {
    #[usage(long, help = "Where to write the PNG; defaults to the pictures directory")]
    pub output: Option<PathBuf>,
    #[usage(
        long,
        help = "Only this part of the frame, as x,y:widthxheight in pixels, e.g. 100,50:800x600"
    )]
    pub region: Option<Region>,
    #[usage(
        long,
        help = "Wait this long in real time first, e.g. 5s, so exposure and other animations settle"
    )]
    pub delay: Option<Duration>,
}

impl Routable for CameraSnapCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::CameraSnap(self))
    }
}
