use bitcode::{Decode, Encode};

use crate::request::{Request, RequestId};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// What a client sends: a request and the id its reply will carry.
pub struct ClientFrame {
    /// The id the reply to this request carries.
    pub id: RequestId,
    /// What the client asks for.
    pub request: Request,
}
