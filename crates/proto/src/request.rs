//! Requests: what a client asks the server to do.
//!
//! A client sends each request in a [`ClientFrame`](crate::frame::ClientFrame) with a
//! [`RequestId`](crate::frame::RequestId). The server
//! checks that the sender has joined and that the request is available in the session, handles it, broadcasts the
//! [`Event`](crate::event::Event)s it caused, then answers only the sender with a
//! [`ServerFrame::Reply`](crate::frame::ServerFrame::Reply): the request's typed [`Reply`](crate::reply::Reply), or a
//! [`RequestError`]. Events come before the reply, so the sender's state already reflects a request when its reply
//! arrives.
//!
//! A request is a `<Name>Request` type in its own file whose [`RequestType`] states its contract, listed in
//! `requests!`.

mod join_request;
mod request_type;
mod set_time_rate_override_request;
mod set_time_rate_request;
mod set_time_request;
mod set_time_warp_request;
mod step_time_rate_request;
mod step_time_request;

pub use join_request::JoinRequest;
pub use request_type::RequestType;
pub use set_time_rate_override_request::SetTimeRateOverrideRequest;
pub use set_time_rate_request::SetTimeRateRequest;
pub use set_time_request::SetTimeRequest;
pub use set_time_warp_request::SetTimeWarpRequest;
pub use step_time_rate_request::StepTimeRateRequest;
pub use step_time_request::StepTimeRequest;

use bitcode::{Decode, Encode};

use crate::error::{ForbiddenCause, RequestError};
use crate::session::{SessionEnvironment, SessionEnvironments};

macro_rules! requests {
    ($($(#[$attribute:meta])* $variant:ident($request:ident)),+ $(,)?) => {
        /// Every request a client can send.
        #[derive(Encode, Decode, Debug, Clone, PartialEq)]
        pub enum Request {
            $($(#[$attribute])* $variant($request)),+
        }

        impl Request {
            /// The kinds of session this request is available in.
            pub fn session_environments(&self) -> SessionEnvironments {
                match self {
                    $(Request::$variant(_) => $request::SESSION_ENVIRONMENTS),+
                }
            }
        }
    };
}

requests! {
    /// See [`JoinRequest`].
    Join(JoinRequest),
    /// See [`SetTimeRequest`].
    SetTime(SetTimeRequest),
    /// See [`StepTimeRequest`].
    StepTime(StepTimeRequest),
    /// See [`SetTimeRateRequest`].
    SetTimeRate(SetTimeRateRequest),
    /// See [`StepTimeRateRequest`].
    StepTimeRate(StepTimeRateRequest),
    /// See [`SetTimeRateOverrideRequest`].
    SetTimeRateOverride(SetTimeRateOverrideRequest),
    /// See [`SetTimeWarpRequest`].
    SetTimeWarp(SetTimeWarpRequest),
}

impl Request {
    /// Refuses the request as [`Forbidden`](RequestError::Forbidden) when it is not available in `environment`.
    pub fn check_session_environment(&self, environment: SessionEnvironment) -> Result<(), RequestError> {
        if self.session_environments().contains(environment) {
            Ok(())
        } else {
            Err(RequestError::Forbidden {
                cause: ForbiddenCause::Unavailable { environment },
            })
        }
    }
}
