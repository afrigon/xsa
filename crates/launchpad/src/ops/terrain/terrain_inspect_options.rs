use std::path::PathBuf;

use resource_loader::terrain::PageAddress;

use crate::terrain::PageCheck;

pub struct TerrainInspectOptions {
    pub file: PathBuf,
    pub check: PageCheck,
    pub page: Option<PageAddress>,
}
