use std::path::PathBuf;

use usage::Args;

use super::DEFAULT_CAPTURE_FPS;

#[derive(Args)]
pub struct CameraCaptureStartCommand {
    #[usage(long, help = "Where to write the MP4; defaults to the videos directory")]
    pub output: Option<PathBuf>,
    #[usage(
        long,
        default_value_t = DEFAULT_CAPTURE_FPS,
        default = "60",
        help = "Frames per second; time advances exactly one frame's worth per frame"
    )]
    pub fps: u32,
}
