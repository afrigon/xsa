mod tick_event;
mod time_changed_event;
mod user_joined_event;
mod user_left_event;

pub use tick_event::TickEvent;
pub use time_changed_event::TimeChangedEvent;
pub use user_joined_event::UserJoinedEvent;
pub use user_left_event::UserLeftEvent;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Event {
    UserJoined(UserJoinedEvent),
    UserLeft(UserLeftEvent),
    TimeChanged(TimeChangedEvent),
    Tick(TickEvent),
}
