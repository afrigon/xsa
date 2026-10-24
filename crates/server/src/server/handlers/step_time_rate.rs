use xsa_proto::error::RequestError;
use xsa_proto::request::StepTimeRateRequest;

use crate::server::Server;
use crate::server::request_handler::RequestHandler;
use crate::world::MAXIMUM_TIME_RATE;

impl RequestHandler for StepTimeRateRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        let rate = self.step.apply(server.world.time_flow().rate);

        if rate > MAXIMUM_TIME_RATE {
            return Err(RequestError::conflict(format!(
                "time already runs at its fastest, {}",
                MAXIMUM_TIME_RATE.multiplier
            )));
        }

        Server::check_time_rate(rate, "time rate")?;
        server.world.set_rate(rate);
        server.broadcast_time();

        Ok(())
    }
}
