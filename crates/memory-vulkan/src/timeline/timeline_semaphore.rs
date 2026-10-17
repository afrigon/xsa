use ash::vk;

use crate::MemoryError;

pub(crate) struct TimelineSemaphore {
    pub semaphore: vk::Semaphore,
    pub submitted: u64,
}

impl TimelineSemaphore {
    pub fn new(device: &ash::Device) -> Result<TimelineSemaphore, MemoryError> {
        let mut type_info = vk::SemaphoreTypeCreateInfo::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);
        let create_info = vk::SemaphoreCreateInfo::default().push_next(&mut type_info);
        let semaphore = unsafe { device.create_semaphore(&create_info, None) }?;

        Ok(TimelineSemaphore {
            semaphore,
            submitted: 0,
        })
    }

    pub fn completed(&self, device: &ash::Device) -> Result<u64, MemoryError> {
        Ok(unsafe { device.get_semaphore_counter_value(self.semaphore) }?)
    }
}
