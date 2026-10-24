use xsa_proto::error::RequestError;
use xsa_proto::request::RequestType;

use super::Server;

pub(super) trait RequestHandler: RequestType {
    fn handle(self, server: &mut Server, client: usize) -> Result<Self::Reply, RequestError>;
}
