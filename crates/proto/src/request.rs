mod join_request;
mod leave_request;
mod request_id;
mod role;
mod set_time_rate_override_request;
mod set_time_rate_request;
mod set_time_request;
mod step_time_rate_request;
mod step_time_request;

pub use join_request::JoinRequest;
pub use leave_request::LeaveRequest;
pub use request_id::RequestId;
pub use role::Role;
pub use set_time_rate_override_request::SetTimeRateOverrideRequest;
pub use set_time_rate_request::SetTimeRateRequest;
pub use set_time_request::SetTimeRequest;
pub use step_time_rate_request::StepTimeRateRequest;
pub use step_time_request::StepTimeRequest;

use bitcode::{Decode, Encode};

use crate::session::{SessionEnvironment, SessionEnvironments};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Request {
    Join(JoinRequest),
    Leave(LeaveRequest),
    SetTime(SetTimeRequest),
    StepTime(StepTimeRequest),
    SetTimeRate(SetTimeRateRequest),
    StepTimeRate(StepTimeRateRequest),
    SetTimeRateOverride(SetTimeRateOverrideRequest),
}

impl Request {
    pub fn session_environment(&self) -> SessionEnvironments {
        match self {
            Request::SetTimeRateOverride(_) => SetTimeRateOverrideRequest::SESSION_ENVIRONMENT,
            Request::Join(_)
            | Request::Leave(_)
            | Request::SetTime(_)
            | Request::StepTime(_)
            | Request::SetTimeRate(_)
            | Request::StepTimeRate(_) => SessionEnvironments::ANY,
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
