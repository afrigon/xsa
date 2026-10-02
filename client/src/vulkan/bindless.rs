use anyhow::ensure;
use ash::vk;

use super::Device;

const SAMPLER_BINDING: u32 = 0;
const CUBE_TEXTURES_BINDING: u32 = 1;
const WRAPPING_SAMPLER_BINDING: u32 = 2;
const TEXTURES_BINDING: u32 = 3;
const MAX_CUBE_TEXTURES: u32 = 64;
const MAX_TEXTURES: u32 = 1024;

pub struct BindlessTextures {
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
    sampler: vk::Sampler,
    wrapping_sampler: vk::Sampler,
    cube_texture_count: u32,
    texture_count: u32,
}

impl BindlessTextures {
    pub fn new(device: &Device) -> anyhow::Result<Self> {
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
        let device_handle = device.handle();
        let sampler = unsafe { device_handle.create_sampler(&sampler_info, None) }?;
        let wrapping_sampler_info = sampler_info.address_mode_u(vk::SamplerAddressMode::REPEAT);
        let wrapping_sampler = unsafe { device_handle.create_sampler(&wrapping_sampler_info, None) }?;

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(CUBE_TEXTURES_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_CUBE_TEXTURES)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(WRAPPING_SAMPLER_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(TEXTURES_BINDING)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_TEXTURES)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];
        let bindless_flags =
            vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND;
        let binding_flags = [
            vk::DescriptorBindingFlags::empty(),
            bindless_flags,
            vk::DescriptorBindingFlags::empty(),
            bindless_flags,
        ];
        let mut binding_flags_info =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&bindings)
            .push_next(&mut binding_flags_info);
        let layout = unsafe { device_handle.create_descriptor_set_layout(&layout_info, None) }?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLER)
                .descriptor_count(2),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_CUBE_TEXTURES + MAX_TEXTURES),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let pool = unsafe { device_handle.create_descriptor_pool(&pool_info, None) }?;

        let layouts = [layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        let set = unsafe { device_handle.allocate_descriptor_sets(&allocate_info) }?[0];

        let sampler_infos = [vk::DescriptorImageInfo::default().sampler(sampler)];
        let wrapping_sampler_infos = [vk::DescriptorImageInfo::default().sampler(wrapping_sampler)];
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
            layout,
            pool,
            set,
            sampler,
            wrapping_sampler,
            cube_texture_count: 0,
            texture_count: 0,
        })
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }

    pub fn set(&self) -> vk::DescriptorSet {
        self.set
    }

    pub fn add_cube(&mut self, device: &Device, view: vk::ImageView) -> anyhow::Result<u32> {
        ensure!(self.cube_texture_count < MAX_CUBE_TEXTURES, "out of cube texture slots");
        let index = self.cube_texture_count;
        self.write_image(device, CUBE_TEXTURES_BINDING, index, view);
        self.cube_texture_count += 1;
        Ok(index)
    }

    pub fn add_texture(&mut self, device: &Device, view: vk::ImageView) -> anyhow::Result<u32> {
        ensure!(self.texture_count < MAX_TEXTURES, "out of texture slots");
        let index = self.texture_count;
        self.write_image(device, TEXTURES_BINDING, index, view);
        self.texture_count += 1;
        Ok(index)
    }

    fn write_image(&self, device: &Device, binding: u32, index: u32, view: vk::ImageView) {
        let image_infos = [vk::DescriptorImageInfo::default()
            .image_view(view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(binding)
            .dst_array_element(index)
            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
            .image_info(&image_infos);
        unsafe { device.handle().update_descriptor_sets(&[write], &[]) };
    }

    pub unsafe fn destroy(&mut self, device: &Device) {
        let device = device.handle();
        unsafe {
            device.destroy_descriptor_pool(self.pool, None);
            device.destroy_descriptor_set_layout(self.layout, None);
            device.destroy_sampler(self.sampler, None);
            device.destroy_sampler(self.wrapping_sampler, None);
        }
    }
}
