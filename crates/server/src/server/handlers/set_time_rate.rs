use xsa_proto::error::RequestError;
use xsa_proto::request::SetTimeRateRequest;

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for SetTimeRateRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        if !self.rate.is_valid() {
            return Err(RequestError::invalid(
                "the time rate must be a finite number of at least 0",
            ));
        }

        server.world.set_rate(self.rate);
        server.broadcast_time();

        Ok(())
    }
}
