use ash::vk;

use crate::GraphImageDescription;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TransientKey {
    pub description: GraphImageDescription,
    pub usage: vk::ImageUsageFlags,
}
