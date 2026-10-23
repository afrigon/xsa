use std::path::PathBuf;

use anyhow::Context;
use usage::Args;
use xsa_client::{ClientOptions, GameClient, RemoteServer};
use xsa_logging::{LogFilter, LogFilterHandle};

use crate::current_build::current_build;
use crate::world_arguments::WorldArguments;

#[derive(Args)]
pub struct ClientArguments {
    #[usage(long, help = "Name xsa ipc reaches this client by; defaults to client-<pid>")]
    pub name: Option<String>,
    #[usage(
        long,
        env_fallback("USER", "USERNAME"),
        default = "player",
        help = "Player name shown to other players"
    )]
    pub player: String,
    #[usage(
        long,
        requires("--fingerprint"),
        help = "Connect to a remote server at <host>:<port>; IPv6 hosts go in brackets, e.g. [::1]:1969"
    )]
    pub remote: Option<String>,
    #[usage(
        long,
        requires("--remote"),
        help = "Certificate fingerprint the remote server printed at startup"
    )]
    pub fingerprint: Option<String>,
    #[usage(flatten)]
    pub world: WorldArguments,
    #[usage(
        long,
        help = "File of commands to run at startup, one per line, each waiting for the previous one"
    )]
    pub script: Option<PathBuf>,
    #[usage(
        long,
        help = "Config file to read and save to; defaults to $XDG_CONFIG_HOME/xsa/config.kdl"
    )]
    pub config: Option<PathBuf>,
}

impl ClientArguments {
    pub fn run(self, log_filter: LogFilterHandle, log_filter_override: Option<LogFilter>) -> anyhow::Result<()> {
        let remote = match self.remote {
            Some(address) => Some(RemoteServer {
                address,
                fingerprint: self.fingerprint.context("--remote requires --fingerprint")?,
            }),
            None => None,
        };
        GameClient::new(ClientOptions {
            build: current_build(),
            name: self.name,
            player: self.player,
            remote,
            world: self.world.into_options(),
            script: self.script,
            config: self.config,
            log_filter,
            log_filter_override,
        })
        .run()
    }
}
