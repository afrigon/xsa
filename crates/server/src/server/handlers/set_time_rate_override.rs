use xsa_proto::request::{RequestError, SetTimeRateOverrideRequest};

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for SetTimeRateOverrideRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        if self.rate.is_some_and(|rate| !rate.is_valid()) {
            return Err(RequestError::invalid(
                "the time override must be a finite number of at least 0",
            ));
        }

        server.world.set_time_override(self.rate);
        server.broadcast_time();

        Ok(())
    }
}
