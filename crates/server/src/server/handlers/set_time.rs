use xsa_proto::request::{RequestError, SetTimeRequest};

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for SetTimeRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        if !self.time.is_valid() {
            return Err(RequestError::invalid("the time must be finite"));
        }

        server.world.set_time(self.time);
        server.broadcast_time();

        Ok(())
    }
}
