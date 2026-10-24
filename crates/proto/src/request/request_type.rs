use crate::reply::Reply;
use crate::session::SessionEnvironments;

/// The contract of a request: what the server replies with and where the request is available.
pub trait RequestType {
    /// What the server replies with when it accepts the request; `()` when it has nothing to return.
    type Reply: Into<Reply>;

    /// The kinds of session the request is available in; elsewhere it is [`Forbidden`](crate::error::RequestError::Forbidden).
    const SESSION_ENVIRONMENTS: SessionEnvironments;
}
