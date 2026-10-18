use std::path::PathBuf;

use crate::terrain::TerrainRegion;

pub struct TerrainExtractOptions {
    pub file: PathBuf,
    pub output: PathBuf,
    pub region: TerrainRegion,
}
