use bitcode::{Decode, Encode};

use super::RequestId;
use crate::error::RequestError;
use crate::event::Event;
use crate::reply::Reply;
use crate::world::WorldState;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// What the server sends, in order on each connection.
pub enum ServerFrame {
    /// The whole world state: sent to a client when it joins, before the events that follow it.
    Snapshot(WorldState),
    /// A fact the server announces to every member.
    Event(Event),
    /// The answer to one request, sent only to its sender after the events the request caused.
    Reply {
        /// The id of the request this answers.
        id: RequestId,
        /// The request's reply, or why it was refused.
        result: Result<Reply, RequestError>,
    },
}
