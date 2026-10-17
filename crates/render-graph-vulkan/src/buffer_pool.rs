use std::collections::HashMap;

use ash::vk;
use gpu_allocator::vulkan::Allocator;

use crate::RenderGraphError;
use crate::buffer_id::BufferId;
use crate::buffer_transient_key::BufferTransientKey;
use crate::declared_buffer::DeclaredBuffer;
use crate::frame_declaration::FrameDeclaration;
use crate::physical_buffer::PhysicalBuffer;

// The buffers the graph allocates. A buffer is never freed while the graph lives, so its BufferId stays valid.
#[derive(Default)]
pub(crate) struct BufferPool {
    buffers: Vec<PhysicalBuffer>,
    transients: HashMap<BufferTransientKey, Vec<BufferId>>,
}

impl BufferPool {
    pub fn buffer(&self, id: BufferId) -> &PhysicalBuffer {
        &self.buffers[id.index]
    }

    pub fn buffer_mut(&mut self, id: BufferId) -> &mut PhysicalBuffer {
        &mut self.buffers[id.index]
    }

    // Buffers with the same description and usage are reused, each by at most one buffer of the frame: the frame's
    // nth buffer of a kind takes the pool's nth buffer of that kind.
    pub fn assign_transients(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        declaration: &FrameDeclaration,
        usage_flags: &[vk::BufferUsageFlags],
    ) -> Result<Vec<Option<BufferId>>, RenderGraphError> {
        let mut used: HashMap<BufferTransientKey, usize> = HashMap::new();
        let mut slots = Vec::with_capacity(declaration.buffers.len());

        for (buffer, usage) in declaration.buffers.iter().zip(usage_flags) {
            let DeclaredBuffer::Transient(description) = buffer else {
                slots.push(None);
                continue;
            };

            if usage.is_empty() {
                slots.push(None);
                continue;
            }

            let key = BufferTransientKey {
                description: *description,
                usage: *usage,
            };
            let nth = used.entry(key).or_default();
            let existing = self.transients.get(&key).and_then(|ids| ids.get(*nth)).copied();
            let id = match existing {
                Some(id) => id,
                None => {
                    let id = self.add(PhysicalBuffer::new(device, allocator, *description, *usage)?);
                    self.transients.entry(key).or_default().push(id);
                    id
                }
            };
            *nth += 1;
            slots.push(Some(id));
        }

        Ok(slots)
    }

    // Safety: the GPU must be done with every buffer.
    pub unsafe fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) -> Result<(), RenderGraphError> {
        self.transients.clear();

        for mut buffer in self.buffers.drain(..) {
            unsafe { buffer.destroy(device, allocator) }?;
        }

        Ok(())
    }

    fn add(&mut self, buffer: PhysicalBuffer) -> BufferId {
        self.buffers.push(buffer);

        BufferId {
            index: self.buffers.len() - 1,
        }
    }
}
