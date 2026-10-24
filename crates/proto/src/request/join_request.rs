use bitcode::{Decode, Encode};

use super::RequestType;
use crate::reply::JoinReply;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Joins the session as a new user.
///
/// The server sends the joining client a [`Snapshot`](crate::frame::ServerFrame::Snapshot) of the world, announces
/// the user to every member with a [`UserJoinedEvent`](crate::event::UserJoinedEvent), then replies with a
/// [`JoinReply`] naming the user. Joining twice is a [`Conflict`](super::RequestError::Conflict).
pub struct JoinRequest;

impl RequestType for JoinRequest {
    type Reply = JoinReply;

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
