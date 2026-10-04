use std::path::PathBuf;

pub struct WorldOptions {
    pub packs_directory: PathBuf,
    pub packs: Vec<String>,
    pub simulation: Option<String>,
}
