use std::error::Error;
use std::fmt;

use ash::vk;
use gpu_allocator::AllocationError;

use crate::PassError;

#[derive(Debug)]
pub enum RenderGraphError {
    /// A pass declared something the graph cannot schedule.
    Declaration {
        pass: &'static str,
        reason: String,
    },
    /// A pass recorded a different number of steps than it declared.
    StepCount {
        pass: &'static str,
        declared: u32,
        recorded: u32,
    },
    Pass {
        pass: &'static str,
        source: PassError,
    },
    Vulkan(vk::Result),
    Allocation(AllocationError),
}

impl fmt::Display for RenderGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderGraphError::Declaration { pass, reason } => write!(formatter, "{pass}: {reason}"),
            RenderGraphError::StepCount {
                pass,
                declared,
                recorded,
            } => write!(formatter, "{pass} declared {declared} steps but recorded {recorded}"),
            RenderGraphError::Pass { pass, .. } => write!(formatter, "recording {pass}"),
            RenderGraphError::Vulkan(result) => write!(formatter, "a Vulkan call failed: {result}"),
            RenderGraphError::Allocation(error) => write!(formatter, "allocating GPU memory: {error}"),
        }
    }
}

impl Error for RenderGraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RenderGraphError::Pass { source, .. } => Some(source.as_ref()),
            RenderGraphError::Vulkan(result) => Some(result),
            RenderGraphError::Allocation(error) => Some(error),
            RenderGraphError::Declaration { .. } | RenderGraphError::StepCount { .. } => None,
        }
    }
}

impl From<vk::Result> for RenderGraphError {
    fn from(result: vk::Result) -> RenderGraphError {
        RenderGraphError::Vulkan(result)
    }
}

impl From<AllocationError> for RenderGraphError {
    fn from(error: AllocationError) -> RenderGraphError {
        RenderGraphError::Allocation(error)
    }
}
