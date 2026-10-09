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
    /// A frame used a history image with other usages than the frame that created it.
    HistoryUsageChanged {
        history: &'static str,
    },
    /// Creating an image failed, then destroying what was already created failed too.
    ImageCleanup {
        image: &'static str,
        creation: Box<RenderGraphError>,
        cleanup: Box<RenderGraphError>,
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
            RenderGraphError::HistoryUsageChanged { history } => {
                write!(
                    formatter,
                    "{history} is used differently than in the frame that created it"
                )
            }
            RenderGraphError::ImageCleanup { image, cleanup, .. } => {
                write!(
                    formatter,
                    "creating {image} failed, then destroying its parts failed: {cleanup}"
                )
            }
            RenderGraphError::Vulkan(result) => write!(formatter, "a Vulkan call failed: {result}"),
            RenderGraphError::Allocation(error) => write!(formatter, "allocating GPU memory: {error}"),
        }
    }
}

impl Error for RenderGraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RenderGraphError::Pass { source, .. } => Some(source.as_ref()),
            RenderGraphError::ImageCleanup { creation, .. } => Some(creation.as_ref()),
            RenderGraphError::Vulkan(result) => Some(result),
            RenderGraphError::Allocation(error) => Some(error),
            RenderGraphError::Declaration { .. }
            | RenderGraphError::StepCount { .. }
            | RenderGraphError::HistoryUsageChanged { .. } => None,
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
