use crate::BufferUsage;

#[derive(Clone, Debug)]
pub struct BufferUsageDescription {
    pub buffer: &'static str,
    pub usage: BufferUsage,
}
