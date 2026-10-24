mod join_request;
mod leave_request;
mod request_id;
mod role;
mod set_time_override_request;
mod set_time_rate_request;
mod set_time_request;
mod step_time_request;

pub use join_request::JoinRequest;
pub use leave_request::LeaveRequest;
pub use request_id::RequestId;
pub use role::Role;
pub use set_time_override_request::SetTimeOverrideRequest;
pub use set_time_rate_request::SetTimeRateRequest;
pub use set_time_request::SetTimeRequest;
pub use step_time_request::StepTimeRequest;

use bitcode::{Decode, Encode};

use crate::session::{SessionEnvironment, SessionEnvironments};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Request {
    Join(JoinRequest),
    Leave(LeaveRequest),
    SetTimeRate(SetTimeRateRequest),
    SetTimeOverride(SetTimeOverrideRequest),
    SetTime(SetTimeRequest),
    StepTime(StepTimeRequest),
}

impl Request {
    pub fn session_environment(&self) -> SessionEnvironments {
        match self {
            Request::SetTimeOverride(_) => SetTimeOverrideRequest::SESSION_ENVIRONMENT,
            Request::Join(_)
            | Request::Leave(_)
            | Request::SetTimeRate(_)
            | Request::SetTime(_)
            | Request::StepTime(_) => SessionEnvironments::ANY,
        }
    }

    pub fn check_session_environment(&self, environment: SessionEnvironment) -> Result<(), String> {
        if self.session_environment().contains(environment) {
            Ok(())
        } else {
            Err(format!("not available in this session ({environment})"))
        }
    }
}
