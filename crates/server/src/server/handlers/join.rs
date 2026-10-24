use xsa_proto::event::{Event, JoinAccepted, Outcome, UserJoinedEvent};
use xsa_proto::request::{JoinRequest, Role};
use xsa_proto::user::{User, UserId};

use crate::server::Server;
use crate::server::membership::Membership;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for JoinRequest {
    fn handle(self, server: &mut Server, client: usize) -> Outcome {
        if server.clients[client].membership.is_some() {
            return Outcome::denied("already joined");
        }

        let membership = match self.role {
            Role::Player { name } => {
                let user = User {
                    id: UserId {
                        value: server.next_user,
                    },
                    name,
                };
                server.next_user += 1;

                Membership::User { user }
            }
            Role::Server => Membership::Server,
        };
        let joined_user = match &membership {
            Membership::User { user } => Some(user.clone()),
            Membership::Server => None,
        };
        server.clients[client].membership = Some(membership);

        let state = server.world.state(server.users(), server.environment);
        server.send(client, Event::JoinAccepted(JoinAccepted { state }));

        if let Some(user) = joined_user {
            server.broadcast_except(Some(client), &Event::UserJoined(UserJoinedEvent { user }));
        }

        Outcome::Accepted
    }
}
