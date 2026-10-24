use xsa_proto::error::RequestError;
use xsa_proto::request::SetTimeRateRequest;

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for SetTimeRateRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        Server::check_time_rate(self.rate, "time rate")?;
        server.world.set_rate(self.rate);
        server.broadcast_time();

        Ok(())
    }
}
