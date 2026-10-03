mod client_frame;
mod join;
mod leave;
mod message_id;
mod role;
mod set_time;
mod set_time_rate;
mod step_time;

pub use client_frame::ClientFrame;
pub use join::Join;
pub use leave::Leave;
pub use message_id::MessageId;
pub use role::Role;
pub use set_time::SetTime;
pub use set_time_rate::SetTimeRate;
pub use step_time::StepTime;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Join(Join),
    Leave(Leave),
    SetTimeRate(SetTimeRate),
    SetTime(SetTime),
    StepTime(StepTime),
}
