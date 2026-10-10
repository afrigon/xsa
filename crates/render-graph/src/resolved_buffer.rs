use ash::vk;

use crate::ImportedBuffer;
use crate::buffer_id::BufferId;
use crate::physical_buffer::PhysicalBuffer;

// What a buffer of this frame is: a pooled buffer, an imported one, or nothing for a buffer no running pass uses.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ResolvedBuffer {
    pub name: &'static str,
    pub buffer: vk::Buffer,
    pub id: Option<BufferId>,
}

impl ResolvedBuffer {
    pub fn unused(name: &'static str) -> ResolvedBuffer {
        ResolvedBuffer {
            name,
            buffer: vk::Buffer::null(),
            id: None,
        }
    }

    pub fn imported(imported: &ImportedBuffer) -> ResolvedBuffer {
        ResolvedBuffer {
            name: imported.name,
            buffer: imported.buffer,
            id: None,
        }
    }

    pub fn physical(id: BufferId, physical: &PhysicalBuffer) -> ResolvedBuffer {
        ResolvedBuffer {
            name: physical.description.name,
            buffer: physical.buffer,
            id: Some(id),
        }
    }
}
