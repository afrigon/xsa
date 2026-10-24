use bitcode::{Decode, Encode};

use crate::request::{Request, RequestId};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct ClientFrame {
    pub id: RequestId,
    pub request: Request,
}
