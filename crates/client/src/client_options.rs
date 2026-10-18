use xsa_server::WorldOptions;

use crate::RemoteServer;

pub struct ClientOptions {
    pub name: Option<String>,
    pub player: String,
    pub remote: Option<RemoteServer>,
    pub world: WorldOptions,
}
