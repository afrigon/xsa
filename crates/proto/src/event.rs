mod join_accepted;
mod join_denied;
mod outcome;
mod pack_reference;
mod player;
mod player_id;
mod player_joined;
mod player_left;
mod reply;
mod tick;
mod time_changed;
mod world_state;

pub use join_accepted::JoinAccepted;
pub use join_denied::JoinDenied;
pub use outcome::Outcome;
pub use pack_reference::PackReference;
pub use player::Player;
pub use player_id::PlayerId;
pub use player_joined::PlayerJoined;
pub use player_left::PlayerLeft;
pub use reply::Reply;
pub use tick::Tick;
pub use time_changed::TimeChanged;
pub use world_state::WorldState;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ServerEvent {
    JoinAccepted(JoinAccepted),
    JoinDenied(JoinDenied),
    PlayerJoined(PlayerJoined),
    PlayerLeft(PlayerLeft),
    TimeChanged(TimeChanged),
    Tick(Tick),
    Reply(Reply),
}
