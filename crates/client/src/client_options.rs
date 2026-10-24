use std::path::PathBuf;

use xsa_logging::{LogFilter, LogFilterHandle};
use xsa_server::WorldOptions;

use crate::{BuildIdentity, RemoteServer};

pub struct ClientOptions {
    pub build: BuildIdentity,
    pub name: Option<String>,
    pub player: String,
    pub remote: Option<RemoteServer>,
    pub world: WorldOptions,
    pub script: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub log_filter: LogFilterHandle,
    pub log_filter_override: Option<LogFilter>,
}
