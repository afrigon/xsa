use ash::khr::video_queue;
use ash::vk;
use memory_vulkan::{Deletion, DeviceHandle};

// A session's parameter sets and the functions that destroy them, so the deletion queue can.
pub(crate) struct VideoSessionParameters {
    pub handle: vk::VideoSessionParametersKHR,
    pub functions: video_queue::Device,
}

impl DeviceHandle for VideoSessionParameters {
    fn into_deletion(self) -> Deletion {
        Deletion::Action(Box::new(move |device| unsafe {
            (self.functions.fp().destroy_video_session_parameters_khr)(device.handle(), self.handle, std::ptr::null());
        }))
    }
}
