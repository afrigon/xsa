use std::path::PathBuf;

use usage::Args;

use super::CameraCaptureAction;
use crate::value::{Duration, Region};

pub const DEFAULT_CAPTURE_FPS: u32 = 60;

#[derive(Args)]
pub struct CameraCaptureCommand {
    #[usage(
        long,
        help = "Where to write the PNG, or the MP4 with --duration; defaults to the pictures or videos directory"
    )]
    pub output: Option<PathBuf>,
    #[usage(
        long,
        help = "Only this part of a snapshot, as x,y:widthxheight in pixels, e.g. 100,50:800x600"
    )]
    pub region: Option<Region>,
    #[usage(long, help = "Capture a video this long instead of a snapshot, e.g. 10s")]
    pub duration: Option<Duration>,
    #[usage(
        long,
        default_value_t = DEFAULT_CAPTURE_FPS,
        default = "60",
        help = "Frames per second of a video; time advances exactly one frame's worth per frame"
    )]
    pub fps: u32,
    #[usage(subcommand)]
    pub action: Option<CameraCaptureAction>,
}
