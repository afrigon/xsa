use bitcode::{Decode, Encode};

use super::RequestType;
use crate::reply::JoinReply;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct JoinRequest;

impl RequestType for JoinRequest {
    type Reply = JoinReply;

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
