use anyhow::Context;
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, AllocatorCreateDesc};

use crate::vulkan::{Device, Instance};

pub struct Allocator {
    allocator: gpu_allocator::vulkan::Allocator,
}

impl Allocator {
    pub fn new(instance: &Instance, device: &Device) -> anyhow::Result<Self> {
        let allocator = gpu_allocator::vulkan::Allocator::new(&AllocatorCreateDesc {
            instance: instance.handle().clone(),
            device: device.handle().clone(),
            physical_device: device.physical_device(),
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })
        .context("creating the GPU memory allocator")?;
        Ok(Self { allocator })
    }

    pub(super) fn allocate(
        &mut self,
        name: &str,
        requirements: vk::MemoryRequirements,
        location: MemoryLocation,
        linear: bool,
    ) -> anyhow::Result<Allocation> {
        let allocation = self.allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        Ok(allocation)
    }

    pub(super) fn free(&mut self, allocation: Allocation) {
        if let Err(err) = self.allocator.free(allocation) {
            tracing::error!("freeing GPU memory: {err}");
        }
    }
}
