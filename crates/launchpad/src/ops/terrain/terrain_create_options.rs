use std::path::PathBuf;

use resource_loader::terrain::PageEncoding;

pub struct TerrainCreateOptions {
    pub input: PathBuf,
    pub output: PathBuf,
    pub encoding: PageEncoding,
}
