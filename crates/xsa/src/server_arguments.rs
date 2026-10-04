use std::net::IpAddr;
use std::path::PathBuf;

use usage::Args;
use xsa_server::{DedicatedOptions, DedicatedServer};

use crate::world_arguments::WorldArguments;

#[derive(Args)]
pub struct ServerArguments {
    #[usage(long, help = "Name for xsa ipc --instance; defaults to server-<pid>")]
    pub instance: Option<String>,
    #[usage(long, default = "::1", help = "Address to listen on")]
    pub host: IpAddr,
    #[usage(long, default = "1969", help = "UDP port to listen on")]
    pub port: u16,
    #[usage(long, default = ".xsa/server", help = "Directory holding the server identity")]
    pub identity: PathBuf,
    #[usage(flatten)]
    pub world: WorldArguments,
}

impl ServerArguments {
    pub fn run(self) -> anyhow::Result<()> {
        DedicatedServer::new(DedicatedOptions {
            instance: self.instance,
            host: self.host,
            port: self.port,
            identity: self.identity,
            world: self.world.into_options(),
        })
        .run()
    }
}
