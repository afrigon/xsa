use std::path::PathBuf;

use anyhow::bail;
use usage::Args;

use super::{Routable, Route};
use crate::script::Script;

#[derive(Args)]
pub struct ScriptCommand {
    #[usage(help = "A file of commands, one per line")]
    pub path: Option<PathBuf>,
    #[usage(
        long,
        help = "Commands separated by semicolons or newlines, e.g. \"input down w; wait 500ms; input up w\""
    )]
    pub text: Option<String>,
}

impl ScriptCommand {
    pub fn script(&self) -> anyhow::Result<Script> {
        match (&self.path, &self.text) {
            (Some(path), None) => Script::load(path),
            (None, Some(text)) => Script::from_text(text),
            _ => bail!("give either a path or --text"),
        }
    }
}

impl Routable for ScriptCommand {
    fn route(self) -> Route {
        Route::Script(self)
    }
}
