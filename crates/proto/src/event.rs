//! Events: facts the server announces.
//!
//! An event says what happened, in the past tense. The server sends it to every member of the session, the client
//! whose request caused it included, and clients apply events to their copy of the session
//! ([`SessionState`](crate::session::SessionState)). A joining client receives a
//! [`Snapshot`](crate::frame::ServerFrame::Snapshot) of the world first, then the events that follow it.
//!
//! An event is a `<Name>Event` type in its own file, named in the past tense.

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
/// Every event the server announces.
pub enum Event {
    /// See [`UserJoinedEvent`].
    UserJoined(UserJoinedEvent),
    /// See [`UserLeftEvent`].
    UserLeft(UserLeftEvent),
    /// See [`TimeChangedEvent`].
    TimeChanged(TimeChangedEvent),
    /// See [`TickEvent`].
    Tick(TickEvent),
}
