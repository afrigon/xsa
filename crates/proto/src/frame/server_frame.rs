use bitcode::{Decode, Encode};

use crate::event::Event;
use crate::reply::Reply;
use crate::request::{RequestError, RequestId};
use crate::world::WorldState;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ServerFrame {
    Snapshot(WorldState),
    Event(Event),
    Reply {
        id: RequestId,
        result: Result<Reply, RequestError>,
    },
}
