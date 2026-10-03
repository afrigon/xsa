use anyhow::Context;
use usage::Args;
use xsa_client::{ClientOptions, GameClient, RemoteServer};

use crate::world_arguments::WorldArguments;

#[derive(Args)]
pub struct ClientArguments {
    #[usage(long, help = "Name for xsa ipc --instance; defaults to client-<pid>")]
    pub instance: Option<String>,
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
}

impl ClientArguments {
    pub fn run(self) -> anyhow::Result<()> {
        let remote = match self.remote {
            Some(address) => Some(RemoteServer {
                address,
                fingerprint: self.fingerprint.context("--remote requires --fingerprint")?,
            }),
            None => None,
        };
        GameClient::new(ClientOptions {
            instance: self.instance,
            player: self.player,
            remote,
            world: self.world.into_options(),
        })
        .run()
    }
}
