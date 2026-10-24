use bitcode::{Decode, Encode};

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct LeaveRequest;

impl RequestType for LeaveRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
