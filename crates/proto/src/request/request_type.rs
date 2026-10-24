use crate::reply::Reply;
use crate::session::SessionEnvironments;

pub trait RequestType {
    type Reply: Into<Reply>;

    const SESSION_ENVIRONMENTS: SessionEnvironments;
}
