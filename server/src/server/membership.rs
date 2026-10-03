use xsa_proto::event::Player;

pub(super) enum Membership {
    Player { player: Player },
    Server,
}
