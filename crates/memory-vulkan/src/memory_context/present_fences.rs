use std::collections::VecDeque;

use ash::vk;

use crate::MemoryError;

// Presents report when the presentation engine has released their swapchain image and semaphores through a fence
// each (VK_EXT_swapchain_maintenance1); presents are counted in the order they were queued.
#[derive(Default)]
pub(crate) struct PresentFences {
    pending: VecDeque<vk::Fence>,
    free: Vec<vk::Fence>,
    queued: u64,
    completed: u64,
}

impl PresentFences {
    pub fn queued(&self) -> u64 {
        self.queued
    }

    pub fn acquire(&mut self, device: &ash::Device) -> Result<vk::Fence, MemoryError> {
        match self.free.pop() {
            Some(fence) => Ok(fence),
            None => Ok(unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }?),
        }
    }

    pub fn queue(&mut self, fence: vk::Fence) {
        self.pending.push_back(fence);
        self.queued += 1;
    }

    // An unused fence, still unsignaled.
    pub fn release(&mut self, fence: vk::Fence) {
        self.free.push(fence);
    }

    // Its present was never queued, so its state is unknown.
    pub fn discard(&mut self, device: &ash::Device, fence: vk::Fence) {
        unsafe { device.destroy_fence(fence, None) };
    }

    // The count of presents completed in order: a later present that completed early waits for the earlier ones.
    pub fn completed(&mut self, device: &ash::Device) -> Result<u64, MemoryError> {
        while let Some(&fence) = self.pending.front()
            && unsafe { device.get_fence_status(fence) }?
        {
            unsafe { device.reset_fences(&[fence]) }?;
            self.pending.pop_front();
            self.free.push(fence);
            self.completed += 1;
        }

        Ok(self.completed)
    }

    pub fn wait_all(&mut self, device: &ash::Device) -> Result<(), MemoryError> {
        let pending: Vec<vk::Fence> = self.pending.iter().copied().collect();

        if !pending.is_empty() {
            unsafe { device.wait_for_fences(&pending, true, u64::MAX) }?;
        }

        self.completed(device)?;

        Ok(())
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        for fence in self.pending.drain(..).chain(self.free.drain(..)) {
            unsafe { device.destroy_fence(fence, None) };
        }
    }
}
