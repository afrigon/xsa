mod join_accepted;
mod join_denied;
mod outcome;
mod reply;
mod tick_event;
mod time_changed_event;
mod user_joined_event;
mod user_left_event;

pub use join_accepted::JoinAccepted;
pub use join_denied::JoinDenied;
pub use outcome::Outcome;
pub use reply::Reply;
pub use tick_event::TickEvent;
pub use time_changed_event::TimeChangedEvent;
pub use user_joined_event::UserJoinedEvent;
pub use user_left_event::UserLeftEvent;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Event {
    JoinAccepted(JoinAccepted),
    JoinDenied(JoinDenied),
    UserJoined(UserJoinedEvent),
    UserLeft(UserLeftEvent),
    TimeChanged(TimeChangedEvent),
    Tick(TickEvent),
    Reply(Reply),
}
