use ash::khr::video_queue;
use ash::vk;
use memory_vulkan::{Deletion, DeviceHandle};

// A video session and the functions that destroy it, so the deletion queue can.
pub(crate) struct VideoSession {
    pub handle: vk::VideoSessionKHR,
    pub functions: video_queue::Device,
}

impl DeviceHandle for VideoSession {
    fn into_deletion(self) -> Deletion {
        Deletion::Action(Box::new(move |device| unsafe {
            (self.functions.fp().destroy_video_session_khr)(device.handle(), self.handle, std::ptr::null());
        }))
    }
}
