use xsa_proto::event::Outcome;
use xsa_proto::message::SetTime;

use crate::server::Server;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for SetTime {
    fn handle(self, server: &mut Server, _client: usize) -> Outcome {
        if !self.time.is_valid() {
            return Outcome::denied("the time must be finite");
        }

        server.world.set_time(self.time);
        server.broadcast_time();

        Outcome::Accepted
    }
}
