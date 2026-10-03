use std::path::PathBuf;

use usage::Args;
use xsa_server::WorldOptions;

#[derive(Args)]
pub struct WorldArguments {
    #[usage(long, default = "data", help = "Directory containing the installed packs")]
    pub packs_directory: PathBuf,
    #[usage(
        long,
        delimiter = ',',
        default = "base",
        default = "system-solar",
        help = "Packs to load, in stack order (later packs override earlier ones)"
    )]
    pub packs: Vec<String>,
    #[usage(
        long,
        help = "Simulation to run, e.g. system-solar:sol; defaults to the only one installed"
    )]
    pub simulation: Option<String>,
}

impl WorldArguments {
    pub fn into_options(self) -> WorldOptions {
        WorldOptions {
            packs_directory: self.packs_directory,
            packs: self.packs,
            simulation: self.simulation,
        }
    }
}
