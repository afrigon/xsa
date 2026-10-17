use std::error::Error;
use std::fmt;
use std::sync::Arc;

use ash::vk;
use gpu_allocator::AllocationError;

use crate::BindlessBinding;

#[derive(Debug)]
pub enum MemoryError {
    Vulkan(vk::Result),
    Allocation(AllocationError),
    /// A buffer the CPU cannot access was read or written, or the access went past its end.
    HostAccess {
        buffer: String,
        reason: &'static str,
    },
    /// A transfer failed along with the others submitted with it, which share its cause.
    Transfer(Arc<MemoryError>),
    /// Every slot of a bindless binding is held.
    OutOfBindlessSlots(BindlessBinding),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemoryError::Vulkan(result) => write!(formatter, "a Vulkan call failed: {result}"),
            MemoryError::Allocation(error) => write!(formatter, "allocating GPU memory: {error}"),
            MemoryError::HostAccess { buffer, reason } => write!(formatter, "accessing {buffer}: {reason}"),
            MemoryError::Transfer(cause) => write!(formatter, "a transfer failed: {cause}"),
            MemoryError::OutOfBindlessSlots(binding) => write!(formatter, "every {binding:?} bindless slot is held"),
        }
    }
}

impl Error for MemoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MemoryError::Vulkan(result) => Some(result),
            MemoryError::Allocation(error) => Some(error),
            MemoryError::HostAccess { .. } | MemoryError::OutOfBindlessSlots(_) => None,
            MemoryError::Transfer(cause) => Some(cause.as_ref()),
        }
    }
}

impl From<vk::Result> for MemoryError {
    fn from(result: vk::Result) -> MemoryError {
        MemoryError::Vulkan(result)
    }
}

impl From<AllocationError> for MemoryError {
    fn from(error: AllocationError) -> MemoryError {
        MemoryError::Allocation(error)
    }
}
