use xsa_proto::request::{RequestError, RequestType};

use super::Server;

pub(super) trait RequestHandler: RequestType {
    fn handle(self, server: &mut Server, client: usize) -> Result<Self::Reply, RequestError>;
}
