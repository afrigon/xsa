use anyhow::ensure;
use ash::vk;
use memory_vulkan::{DeviceObject, MemoryContext};

use super::Device;

const SAMPLER_BINDING: u32 = 0;
const CUBE_TEXTURES_BINDING: u32 = 1;
const WRAPPING_SAMPLER_BINDING: u32 = 2;
const TEXTURES_BINDING: u32 = 3;
const STORAGE_IMAGES_BINDING: u32 = 4;
const MAX_CUBE_TEXTURES: u32 = 64;
const MAX_TEXTURES: u32 = 1024;
const MAX_STORAGE_IMAGES: u32 = 64;

pub const SAMPLED_LAYOUT: vk::ImageLayout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

struct ImageWrite {
    binding: u32,
    index: u32,
    view: vk::ImageView,
    descriptor_type: vk::DescriptorType,
    layout: vk::ImageLayout,
}

impl ImageWrite {
    fn sampled(binding: u32, index: u32, view: vk::ImageView, layout: vk::ImageLayout) -> Self {
        Self {
            binding,
            index,
            view,
            descriptor_type: vk::DescriptorType::SAMPLED_IMAGE,
            layout,
        }
    }
}

pub struct BindlessTextures {
    // Declaration order is destruction order: the pool frees its set before the layout and samplers it uses.
    _pool: DeviceObject<vk::DescriptorPool>,
    set: vk::DescriptorSet,
    layout: DeviceObject<vk::DescriptorSetLayout>,
    _sampler: DeviceObject<vk::Sampler>,
    _wrapping_sampler: DeviceObject<vk::Sampler>,
    cube_texture_count: u32,
    texture_count: u32,
    storage_image_count: u32,
}

impl BindlessTextures {
    pub fn new(memory: &MemoryContext, device: &Device) -> anyhow::Result<Self> {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(true)
            .max_anisotropy(device.max_sampler_anisotropy())
            .max_lod(vk::LOD_CLAMP_NONE);
        let device_handle = memory.device();
        let sampler = DeviceObject::new(memory, unsafe { device_handle.create_sampler(&sampler_info, None) }?);
        let wrapping_sampler_info = sampler_info.address_mode_u(vk::SamplerAddressMode::REPEAT);
        let wrapping_sampler = DeviceObject::new(memory, unsafe {
            device_handle.create_sampler(&wrapping_sampler_info, None)
        }?);

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(CUBE_TEXTURES_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_CUBE_TEXTURES)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(WRAPPING_SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE,
                ),
            vk::DescriptorSetLayoutBinding::default()
                .binding(TEXTURES_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_TEXTURES)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE,
                ),
            vk::DescriptorSetLayoutBinding::default()
                .binding(STORAGE_IMAGES_BINDING)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(MAX_STORAGE_IMAGES)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let bindless_flags =
            vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND;
        let binding_flags = [
            vk::DescriptorBindingFlags::empty(),
            bindless_flags,
            vk::DescriptorBindingFlags::empty(),
            bindless_flags,
            bindless_flags,
        ];
        let mut binding_flags_info =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&bindings)
            .push_next(&mut binding_flags_info);
        let layout = DeviceObject::new(memory, unsafe {
            device_handle.create_descriptor_set_layout(&layout_info, None)
        }?);

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLER)
                .descriptor_count(2),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_CUBE_TEXTURES + MAX_TEXTURES),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(MAX_STORAGE_IMAGES),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let pool = DeviceObject::new(memory, unsafe {
            device_handle.create_descriptor_pool(&pool_info, None)
        }?);

        let layouts = [layout.handle()];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool.handle())
            .set_layouts(&layouts);
        let set = unsafe { device_handle.allocate_descriptor_sets(&allocate_info) }?[0];

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
        unsafe { device_handle.update_descriptor_sets(&writes, &[]) };

        Ok(Self {
            _pool: pool,
            set,
            layout,
            _sampler: sampler,
            _wrapping_sampler: wrapping_sampler,
            cube_texture_count: 0,
            texture_count: 0,
            storage_image_count: 0,
        })
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout.handle()
    }

    pub fn set(&self) -> vk::DescriptorSet {
        self.set
    }

    // A slot no shader reads until `set_cube` fills it.
    pub fn reserve_cube(&mut self) -> anyhow::Result<u32> {
        ensure!(self.cube_texture_count < MAX_CUBE_TEXTURES, "out of cube texture slots");
        let index = self.cube_texture_count;
        self.cube_texture_count += 1;
        Ok(index)
    }

    pub fn set_cube(&self, device: &Device, index: u32, view: vk::ImageView) {
        self.write_image(
            device,
            &ImageWrite::sampled(CUBE_TEXTURES_BINDING, index, view, SAMPLED_LAYOUT),
        );
    }

    pub fn add_texture(
        &mut self,
        device: &Device,
        view: vk::ImageView,
        layout: vk::ImageLayout,
    ) -> anyhow::Result<u32> {
        ensure!(self.texture_count < MAX_TEXTURES, "out of texture slots");
        let index = self.texture_count;
        self.set_texture(device, index, view, layout);
        self.texture_count += 1;
        Ok(index)
    }

    // A slot no shader reads until `set_texture` fills it.
    pub fn reserve_texture(&mut self) -> anyhow::Result<u32> {
        ensure!(self.texture_count < MAX_TEXTURES, "out of texture slots");
        let index = self.texture_count;
        self.texture_count += 1;
        Ok(index)
    }

    pub fn set_texture(&self, device: &Device, index: u32, view: vk::ImageView, layout: vk::ImageLayout) {
        self.write_image(device, &ImageWrite::sampled(TEXTURES_BINDING, index, view, layout));
    }

    pub fn add_storage_image(&mut self, device: &Device, view: vk::ImageView) -> anyhow::Result<u32> {
        ensure!(
            self.storage_image_count < MAX_STORAGE_IMAGES,
            "out of storage image slots"
        );
        let index = self.storage_image_count;
        self.set_storage_image(device, index, view);
        self.storage_image_count += 1;
        Ok(index)
    }

    pub fn set_storage_image(&self, device: &Device, index: u32, view: vk::ImageView) {
        self.write_image(
            device,
            &ImageWrite {
                binding: STORAGE_IMAGES_BINDING,
                index,
                view,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                layout: vk::ImageLayout::GENERAL,
            },
        );
    }

    fn write_image(&self, device: &Device, image: &ImageWrite) {
        let image_infos = [vk::DescriptorImageInfo::default()
            .image_view(image.view)
            .image_layout(image.layout)];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(image.binding)
            .dst_array_element(image.index)
            .descriptor_type(image.descriptor_type)
            .image_info(&image_infos);
        unsafe { device.handle().update_descriptor_sets(&[write], &[]) };
    }
}
