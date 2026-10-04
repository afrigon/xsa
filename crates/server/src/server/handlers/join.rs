use xsa_proto::event::{JoinAccepted, Outcome, Player, PlayerId, PlayerJoined, ServerEvent};
use xsa_proto::message::{Join, Role};

use crate::server::Server;
use crate::server::membership::Membership;
use crate::server::message_handler::MessageHandler;

impl MessageHandler for Join {
    fn handle(self, server: &mut Server, client: usize) -> Outcome {
        if server.clients[client].membership.is_some() {
            return Outcome::denied("already joined");
        }

        let membership = match self.role {
            Role::Player { name } => {
                let player = Player {
                    id: PlayerId {
                        value: server.next_player,
                    },
                    name,
                };
                server.next_player += 1;

                Membership::Player { player }
            }
            Role::Server => Membership::Server,
        };
        let joined_player = match &membership {
            Membership::Player { player } => Some(player.clone()),
            Membership::Server => None,
        };
        server.clients[client].membership = Some(membership);

        let state = server.world.state(server.players());
        server.send(client, ServerEvent::JoinAccepted(JoinAccepted { state }));

        if let Some(player) = joined_player {
            server.broadcast_except(Some(client), &ServerEvent::PlayerJoined(PlayerJoined { player }));
        }

        Outcome::Accepted
    }
}
