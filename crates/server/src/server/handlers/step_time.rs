use xsa_proto::event::Outcome;
use xsa_proto::message::StepTime;

use crate::server::Server;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for StepTime {
    fn handle(self, server: &mut Server, _client: usize) -> Outcome {
        if !self.duration.is_valid_step() {
            return Outcome::denied("the step must be a finite duration greater than 0");
        }

        server.world.step(self.duration);
        server.broadcast_time();

        Outcome::Accepted
    }
}
