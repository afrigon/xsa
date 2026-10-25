use std::net::IpAddr;
use std::path::PathBuf;

use xsa_logging::{LogFilter, LogFilterHandle};

use crate::WorldOptions;

pub struct DedicatedOptions {
    pub name: Option<String>,
    pub host: IpAddr,
    pub port: u16,
    pub identity: PathBuf,
    pub config: PathBuf,
    pub script: Option<PathBuf>,
    pub log_filter: LogFilterHandle,
    pub log_filter_override: Option<LogFilter>,
    pub world: WorldOptions,
}
