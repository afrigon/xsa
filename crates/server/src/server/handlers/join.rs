use xsa_proto::event::{Event, UserJoinedEvent};
use xsa_proto::reply::JoinReply;
use xsa_proto::request::{JoinRequest, RequestError};
use xsa_proto::user::{Actor, User, UserId};

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for JoinRequest {
    fn handle(self, server: &mut Server, client: usize) -> Result<JoinReply, RequestError> {
        if server.clients[client].actor.is_some() {
            return Err(RequestError::conflict("already joined"));
        }

        let user = User {
            id: UserId {
                value: server.next_user,
            },
        };
        server.next_user += 1;

        server.send_snapshot(client);
        server.users.push(user.clone());
        server.clients[client].actor = Some(Actor::User(user.id));
        server.broadcast(&Event::UserJoined(UserJoinedEvent { user: user.clone() }));

        Ok(JoinReply { user: user.id })
    }
}
