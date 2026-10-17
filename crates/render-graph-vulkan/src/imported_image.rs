use ash::vk;

use crate::ImageState;

/// An image owned outside the graph, such as a swapchain image, and the states it starts and must end the frame in.
#[derive(Clone, Copy, Debug)]
pub struct ImportedImage {
    pub name: &'static str,
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub extent: vk::Extent2D,
    pub aspect: vk::ImageAspectFlags,
    pub initial: ImageState,
    pub final_state: ImageState,
}
