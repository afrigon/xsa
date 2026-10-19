mod bindless_binding;
mod bindless_slot;
mod free_slots;

pub use bindless_binding::BindlessBinding;
pub use bindless_slot::BindlessSlot;

pub(crate) use free_slots::FreeSlots;

use std::sync::{Arc, Mutex};

use ash::vk;

use crate::{DeviceObject, MemoryContext, MemoryError};

const SAMPLER_BINDING: u32 = 0;
const WRAPPING_SAMPLER_BINDING: u32 = 2;
const SAMPLER_COUNT: u32 = 2;

/// The descriptor set every shader reads images through: two samplers, then arrays of textures, cube maps and storage
/// images, each image at the index of the slot it was written into.
///
/// Cloning it is cheap, and clones share the table across threads.
#[derive(Clone)]
pub struct BindlessTable {
    shared: Arc<SharedTable>,
}

struct SharedTable {
    // Declaration order is destruction order: the pool frees its set before the layout and samplers it uses.
    _pool: DeviceObject<vk::DescriptorPool>,
    set: vk::DescriptorSet,
    layout: DeviceObject<vk::DescriptorSetLayout>,
    _sampler: DeviceObject<vk::Sampler>,
    _wrapping_sampler: DeviceObject<vk::Sampler>,
    free: Arc<Mutex<FreeSlots>>,
    // Updating a descriptor set must be externally synchronized.
    writes: Mutex<()>,
    memory: MemoryContext,
}

impl BindlessTable {
    /// The device must enable runtime descriptor arrays, partially bound descriptors, and updating sampled and storage
    /// images after bind and while pending.
    pub fn new(memory: &MemoryContext, max_anisotropy: f32) -> Result<BindlessTable, MemoryError> {
        let device = memory.device();
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(true)
            .max_anisotropy(max_anisotropy)
            .max_lod(vk::LOD_CLAMP_NONE);
        let sampler = DeviceObject::new(memory, unsafe { device.create_sampler(&sampler_info, None) }?);
        let wrapping_sampler_info = sampler_info.address_mode_u(vk::SamplerAddressMode::REPEAT);
        let wrapping_sampler =
            DeviceObject::new(memory, unsafe { device.create_sampler(&wrapping_sampler_info, None) }?);

        let all_stages = vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE;
        let image_stages = |binding: BindlessBinding| match binding {
            BindlessBinding::Texture => all_stages,
            BindlessBinding::CubeMap => vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE,
            BindlessBinding::StorageImage => vk::ShaderStageFlags::COMPUTE,
        };
        let mut bindings = vec![
            vk::DescriptorSetLayoutBinding::default()
                .binding(SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(WRAPPING_SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(all_stages),
        ];
        let mut binding_flags = vec![vk::DescriptorBindingFlags::empty(); bindings.len()];

        for binding in BindlessBinding::ALL {
            bindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(binding.binding())
                    .descriptor_type(binding.descriptor_type())
                    .descriptor_count(binding.capacity())
                    .stage_flags(image_stages(binding)),
            );
            binding_flags.push(
                vk::DescriptorBindingFlags::PARTIALLY_BOUND
                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
            );
        }

        let mut binding_flags_info =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&bindings)
            .push_next(&mut binding_flags_info);
        let layout = DeviceObject::new(memory, unsafe {
            device.create_descriptor_set_layout(&layout_info, None)
        }?);

        let sampled_count = BindlessBinding::Texture.capacity() + BindlessBinding::CubeMap.capacity();
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLER)
                .descriptor_count(SAMPLER_COUNT),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(sampled_count),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(BindlessBinding::StorageImage.capacity()),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let pool = DeviceObject::new(memory, unsafe { device.create_descriptor_pool(&pool_info, None) }?);

        let layouts = [layout.handle()];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool.handle())
            .set_layouts(&layouts);
        let set = unsafe { device.allocate_descriptor_sets(&allocate_info) }?[0];

        let sampler_infos = [vk::DescriptorImageInfo::default().sampler(sampler.handle())];
        let wrapping_sampler_infos = [vk::DescriptorImageInfo::default().sampler(wrapping_sampler.handle())];
        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .image_info(&sampler_infos),
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(WRAPPING_SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .image_info(&wrapping_sampler_infos),
        ];
        unsafe { device.update_descriptor_sets(&writes, &[]) };

        Ok(BindlessTable {
            shared: Arc::new(SharedTable {
                _pool: pool,
                set,
                layout,
                _sampler: sampler,
                _wrapping_sampler: wrapping_sampler,
                free: Arc::new(Mutex::new(FreeSlots::default())),
                writes: Mutex::new(()),
                memory: memory.clone(),
            }),
        })
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.shared.layout.handle()
    }

    pub fn set(&self) -> vk::DescriptorSet {
        self.shared.set
    }

    /// A slot no shader reads until `write` fills it.
    pub fn allocate(&self, binding: BindlessBinding) -> Result<BindlessSlot, MemoryError> {
        let index = self
            .shared
            .free
            .lock()
            .expect("the bindless free slots' lock is poisoned")
            .take(binding)
            .ok_or(MemoryError::OutOfBindlessSlots(binding))?;

        Ok(BindlessSlot::new(
            binding,
            index,
            Arc::clone(&self.shared.free),
            self.shared.memory.clone(),
        ))
    }

    /// Points `slot` at `view`, in `layout` whenever a shader reads it. A slot written for the first time may be
    /// written while frames are in flight; rewriting one a pending frame reads is not allowed.
    pub fn write(&self, slot: &BindlessSlot, view: vk::ImageView, layout: vk::ImageLayout) {
        let image_infos = [vk::DescriptorImageInfo::default().image_view(view).image_layout(layout)];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.shared.set)
            .dst_binding(slot.binding().binding())
            .dst_array_element(slot.index())
            .descriptor_type(slot.binding().descriptor_type())
            .image_info(&image_infos);
        let _writing = self
            .shared
            .writes
            .lock()
            .expect("the bindless writes' lock is poisoned");
        unsafe { self.shared.memory.device().update_descriptor_sets(&[write], &[]) };
    }
}
