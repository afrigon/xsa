use std::net::IpAddr;
use std::path::PathBuf;

use crate::WorldOptions;

pub struct DedicatedOptions {
    pub name: Option<String>,
    pub host: IpAddr,
    pub port: u16,
    pub identity: PathBuf,
    pub world: WorldOptions,
}
