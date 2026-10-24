mod forbidden_cause;
mod join_request;
mod leave_request;
mod request_error;
mod request_id;
mod request_type;
mod set_time_rate_override_request;
mod set_time_rate_request;
mod set_time_request;
mod step_time_rate_request;
mod step_time_request;

pub use forbidden_cause::ForbiddenCause;
pub use join_request::JoinRequest;
pub use leave_request::LeaveRequest;
pub use request_error::RequestError;
pub use request_id::RequestId;
pub use request_type::RequestType;
pub use set_time_rate_override_request::SetTimeRateOverrideRequest;
pub use set_time_rate_request::SetTimeRateRequest;
pub use set_time_request::SetTimeRequest;
pub use step_time_rate_request::StepTimeRateRequest;
pub use step_time_request::StepTimeRequest;

use bitcode::{Decode, Encode};

use crate::session::{SessionEnvironment, SessionEnvironments};

macro_rules! requests {
    ($($(#[$attribute:meta])* $variant:ident($request:ident)),+ $(,)?) => {
        #[derive(Encode, Decode, Debug, Clone, PartialEq)]
        pub enum Request {
            $($(#[$attribute])* $variant($request)),+
        }

        impl Request {
            pub fn session_environments(&self) -> SessionEnvironments {
                match self {
                    $(Request::$variant(_) => $request::SESSION_ENVIRONMENTS),+
                }
            }
        }
    };
}

requests! {
    Join(JoinRequest),
    Leave(LeaveRequest),
    SetTime(SetTimeRequest),
    StepTime(StepTimeRequest),
    SetTimeRate(SetTimeRateRequest),
    StepTimeRate(StepTimeRateRequest),
    SetTimeRateOverride(SetTimeRateOverrideRequest),
}

impl Request {
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
