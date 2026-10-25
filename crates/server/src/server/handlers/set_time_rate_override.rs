use xsa_proto::error::RequestError;
use xsa_proto::request::SetTimeRateOverrideRequest;

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for SetTimeRateOverrideRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        if let Some(rate) = self.rate {
            Server::check_time_rate(rate, "time override")?;
        }

        server.world.set_time_override(self.rate);
        server.broadcast_time();

        Ok(())
    }
}
