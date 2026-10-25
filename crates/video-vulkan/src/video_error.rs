use std::fmt;

use ash::vk;
use memory_vulkan::MemoryError;

/// Why video encoding failed.
#[derive(Debug)]
pub enum VideoError {
    /// A Vulkan call failed.
    Vulkan(vk::Result),
    /// Creating or freeing a GPU object failed.
    Memory(MemoryError),
    /// The GPU cannot encode what was asked of it.
    Unsupported(String),
    /// The encoder failed a frame, reporting this query status.
    Encoding(i32),
    /// Writing the video file failed.
    Io(std::io::Error),
}

impl fmt::Display for VideoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoError::Vulkan(result) => write!(formatter, "Vulkan error: {result}"),
            VideoError::Memory(error) => write!(formatter, "{error}"),
            VideoError::Unsupported(reason) => write!(formatter, "unsupported: {reason}"),
            VideoError::Encoding(status) => write!(formatter, "the encoder failed a frame with status {status}"),
            VideoError::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for VideoError {}

impl From<vk::Result> for VideoError {
    fn from(result: vk::Result) -> VideoError {
        VideoError::Vulkan(result)
    }
}

impl From<MemoryError> for VideoError {
    fn from(error: MemoryError) -> VideoError {
        VideoError::Memory(error)
    }
}

impl From<std::io::Error> for VideoError {
    fn from(error: std::io::Error) -> VideoError {
        VideoError::Io(error)
    }
}
