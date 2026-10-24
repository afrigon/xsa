use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};

use crate::MemoryError;

// The allocator, counting the device-local bytes its blocks hold and the bytes its allocations occupy in them. The
// driver counts whole blocks, so freeing an allocation from a shared block lowers only the second.
pub(crate) struct CountedAllocator {
    allocator: Allocator,
    device_local_blocks: u64,
    device_local_allocations: u64,
}

impl CountedAllocator {
    pub fn new(allocator: Allocator) -> CountedAllocator {
        CountedAllocator {
            allocator,
            device_local_blocks: 0,
            device_local_allocations: 0,
        }
    }

    pub fn allocate(&mut self, description: &AllocationCreateDesc) -> Result<Allocation, MemoryError> {
        let capacity = self.allocator.capacity();
        let allocation = self.allocator.allocate(description)?;

        if CountedAllocator::is_device_local(&allocation) {
            self.device_local_blocks += self.allocator.capacity() - capacity;
            self.device_local_allocations += allocation.size();
        }

        Ok(allocation)
    }

    pub fn free(&mut self, allocation: Allocation) -> Result<(), MemoryError> {
        let device_local = CountedAllocator::is_device_local(&allocation);
        let size = allocation.size();
        let capacity = self.allocator.capacity();
        self.allocator.free(allocation)?;

        if device_local {
            self.device_local_blocks -= capacity - self.allocator.capacity();
            self.device_local_allocations -= size;
        }

        Ok(())
    }

    // Device-local bytes the allocator's blocks hold that no allocation occupies.
    pub fn unallocated_device_local(&self) -> u64 {
        self.device_local_blocks - self.device_local_allocations
    }

    fn is_device_local(allocation: &Allocation) -> bool {
        allocation
            .memory_properties()
            .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
    }
}
