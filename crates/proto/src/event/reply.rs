use bitcode::{Decode, Encode};

use super::Outcome;
use crate::request::RequestId;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Reply {
    pub id: RequestId,
    pub outcome: Outcome,
}
