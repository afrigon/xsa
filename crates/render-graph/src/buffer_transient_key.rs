use ash::vk;

use crate::GraphBufferDescription;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BufferTransientKey {
    pub description: GraphBufferDescription,
    pub usage: vk::BufferUsageFlags,
}
