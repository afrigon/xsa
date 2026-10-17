use std::error::Error;
use std::fmt;

use memory_vulkan::MemoryError;

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
    Memory(MemoryError),
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
            RenderGraphError::Memory(_) => write!(formatter, "creating a graph image or buffer"),
        }
    }
}

impl Error for RenderGraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RenderGraphError::Pass { source, .. } => Some(source.as_ref()),
            RenderGraphError::Memory(error) => Some(error),
            RenderGraphError::Declaration { .. }
            | RenderGraphError::StepCount { .. }
            | RenderGraphError::HistoryUsageChanged { .. } => None,
        }
    }
}

impl From<MemoryError> for RenderGraphError {
    fn from(error: MemoryError) -> RenderGraphError {
        RenderGraphError::Memory(error)
    }
}
