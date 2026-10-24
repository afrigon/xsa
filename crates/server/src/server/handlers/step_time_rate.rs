use xsa_proto::request::{RequestError, StepTimeRateRequest};

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for StepTimeRateRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        let rate = self.step.apply(server.world.time_flow().rate);

        if !rate.is_valid() {
            return Err(RequestError::invalid(
                "the time rate must be a finite number of at least 0",
            ));
        }

        server.world.set_rate(rate);
        server.broadcast_time();

        Ok(())
    }
}
