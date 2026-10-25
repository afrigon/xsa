//! Errors: why the server refused a request.

mod forbidden_cause;
mod request_error;

pub use forbidden_cause::ForbiddenCause;
pub use request_error::RequestError;
