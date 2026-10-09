use crate::{BufferUsageDescription, ImageUsageDescription};

#[derive(Clone, Debug)]
pub struct StepDescription {
    pub image_usages: Vec<ImageUsageDescription>,
    pub buffer_usages: Vec<BufferUsageDescription>,
    /// Barriers the graph records before the step, before levels left in differing states are split.
    pub barriers: usize,
}
