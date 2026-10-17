use ash::vk;

use crate::GraphImageDescription;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ImageTransientKey {
    pub description: GraphImageDescription,
    pub usage: vk::ImageUsageFlags,
}
