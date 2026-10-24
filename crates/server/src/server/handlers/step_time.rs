use xsa_proto::request::{RequestError, StepTimeRequest};

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for StepTimeRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        if !self.duration.is_valid_step() {
            return Err(RequestError::invalid(
                "the step must be a finite duration greater than 0",
            ));
        }

        server.world.step(self.duration);
        server.broadcast_time();

        Ok(())
    }
}
