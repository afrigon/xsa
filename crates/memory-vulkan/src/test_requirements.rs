use std::ffi::CStr;

use ash::vk;

/// What a test needs from a `TestDevice` beyond what the crate's types require.
#[derive(Clone, Debug, Default)]
pub struct TestRequirements {
    /// Device extensions to enable.
    pub extensions: Vec<&'static CStr>,
    /// The capabilities of one more queue to create, from a family other than the graphics and transfer ones.
    pub queue: Option<vk::QueueFlags>,
}
