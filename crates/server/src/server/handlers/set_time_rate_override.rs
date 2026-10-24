use xsa_proto::event::Outcome;
use xsa_proto::request::SetTimeRateOverrideRequest;

use crate::server::Server;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for SetTimeRateOverrideRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Outcome {
        if self.rate.is_some_and(|rate| !rate.is_valid()) {
            return Outcome::denied("the time override must be a finite number of at least 0");
        }

        server.world.set_time_override(self.rate);
        server.broadcast_time();

        Outcome::Accepted
    }
}
