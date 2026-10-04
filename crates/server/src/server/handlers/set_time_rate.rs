use xsa_proto::event::Outcome;
use xsa_proto::message::SetTimeRate;

use crate::server::Server;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for SetTimeRate {
    fn handle(self, server: &mut Server, _client: usize) -> Outcome {
        if !self.rate.is_valid() {
            return Outcome::denied("the time rate must be a finite number of at least 0");
        }

        server.world.set_rate(self.rate);
        server.broadcast_time();

        Outcome::Accepted
    }
}
