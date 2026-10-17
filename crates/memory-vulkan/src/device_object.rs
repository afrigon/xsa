use crate::{DeviceHandle, MemoryContext};

/// Owns one object created from the device. Dropping it queues the object's destruction until the GPU has finished
/// all work submitted so far.
pub struct DeviceObject<H: DeviceHandle> {
    handle: Option<H>,
    memory: MemoryContext,
}

impl<H: DeviceHandle> DeviceObject<H> {
    /// Takes ownership of a handle just created from `memory`'s device.
    pub fn new(memory: &MemoryContext, handle: H) -> DeviceObject<H> {
        DeviceObject {
            handle: Some(handle),
            memory: memory.clone(),
        }
    }

    pub fn get(&self) -> &H {
        self.handle
            .as_ref()
            .expect("a device object holds its handle until dropped")
    }

    pub fn get_mut(&mut self) -> &mut H {
        self.handle
            .as_mut()
            .expect("a device object holds its handle until dropped")
    }
}

impl<H: DeviceHandle + Copy> DeviceObject<H> {
    pub fn handle(&self) -> H {
        *self.get()
    }
}

impl<H: DeviceHandle> Drop for DeviceObject<H> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.memory.defer(handle.into_deletion());
        }
    }
}
