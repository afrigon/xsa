use crate::BufferUsage;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DeclaredBufferUsage {
    pub pass: usize,
    pub step: u32,
    pub buffer: usize,
    pub usage: BufferUsage,
}
