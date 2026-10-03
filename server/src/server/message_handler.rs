use xsa_proto::event::Outcome;

use super::Server;

pub(super) trait MessageHandler {
    fn handle(self, server: &mut Server, client: usize) -> Outcome;
}
