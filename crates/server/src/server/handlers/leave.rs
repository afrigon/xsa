use xsa_proto::event::Outcome;
use xsa_proto::request::LeaveRequest;

use crate::server::Server;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for LeaveRequest {
    fn handle(self, server: &mut Server, client: usize) -> Outcome {
        server.clients[client].connected = false;

        Outcome::Accepted
    }
}
