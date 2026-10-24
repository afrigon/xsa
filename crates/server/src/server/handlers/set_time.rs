use xsa_proto::event::Outcome;
use xsa_proto::request::SetTimeRequest;

use crate::server::Server;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for SetTimeRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Outcome {
        if !self.time.is_valid() {
            return Outcome::denied("the time must be finite");
        }

        server.world.set_time(self.time);
        server.broadcast_time();

        Outcome::Accepted
    }
}
