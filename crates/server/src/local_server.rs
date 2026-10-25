use xsa_proto::network::Connection;

use crate::ServerHandle;

pub struct LocalServer {
    pub connection: Connection,
    pub handle: ServerHandle,
}
