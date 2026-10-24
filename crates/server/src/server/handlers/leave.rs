use xsa_proto::request::{LeaveRequest, RequestError};

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for LeaveRequest {
    fn handle(self, server: &mut Server, client: usize) -> Result<(), RequestError> {
        server.clients[client].connected = false;

        Ok(())
    }
}
