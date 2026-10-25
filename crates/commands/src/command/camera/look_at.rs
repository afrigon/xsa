use usage::Args;

use crate::completion::complete_body;

#[derive(Args)]
pub struct CameraLookAtCommand {
    #[usage(complete = complete_body, help = "The body to turn towards, by id")]
    pub body: String,
}
