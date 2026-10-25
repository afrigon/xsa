use std::net::IpAddr;
use std::path::PathBuf;

use usage::Args;
use xsa_logging::{LogFilter, LogFilterHandle};
use xsa_server::{DedicatedOptions, DedicatedServer};

use crate::world_arguments::WorldArguments;

#[derive(Args)]
pub struct ServerArguments {
    #[usage(long, help = "Name xsa ipc reaches this server by; defaults to server-<pid>")]
    pub name: Option<String>,
    #[usage(long, default = "::1", help = "Address to listen on")]
    pub host: IpAddr,
    #[usage(long, default = "1969", help = "UDP port to listen on")]
    pub port: u16,
    #[usage(long, default = ".xsa/server", help = "Directory holding the server identity")]
    pub identity: PathBuf,
    #[usage(long, default = ".xsa/server/config.kdl", help = "Config file to read and save to")]
    pub config: PathBuf,
    #[usage(
        long,
        help = "File of commands to run at startup, one per line, each waiting for the previous one"
    )]
    pub script: Option<PathBuf>,
    #[usage(flatten)]
    pub world: WorldArguments,
}

impl ServerArguments {
    pub fn run(self, log_filter: LogFilterHandle, log_filter_override: Option<LogFilter>) -> anyhow::Result<()> {
        DedicatedServer::new(DedicatedOptions {
            name: self.name,
            host: self.host,
            port: self.port,
            identity: self.identity,
            config: self.config,
            script: self.script,
            log_filter,
            log_filter_override,
            world: self.world.into_options(),
        })
        .run()
    }
}
