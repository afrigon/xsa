use std::io::Cursor;

use ash::vk;
use memory_vulkan::{DeviceObject, MemoryContext};

use crate::push_constants::PushConstants;

const SHADER: &[u8] = include_bytes!("../shaders/xui.spv");

pub(crate) struct InterfacePipeline {
    descriptor_pool: DeviceObject<vk::DescriptorPool>,
    sampler: DeviceObject<vk::Sampler>,
    pipeline: DeviceObject<vk::Pipeline>,
    layout: DeviceObject<vk::PipelineLayout>,
    set_layout: DeviceObject<vk::DescriptorSetLayout>,
}

impl InterfacePipeline {
    pub fn new(
        memory: &MemoryContext,
        color_format: vk::Format,
        frames_in_flight: u32,
    ) -> anyhow::Result<InterfacePipeline> {
        let device = memory.device();
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
        ];
        let set_layout = DeviceObject::new(memory, unsafe {
            device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None)
        }?);
        let set_layouts = [set_layout.handle()];
        let push_constant_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .size(size_of::<PushConstants>() as u32)];
        let layout = DeviceObject::new(memory, unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&set_layouts)
                    .push_constant_ranges(&push_constant_ranges),
                None,
            )
        }?);
        let pipeline = InterfacePipeline::create_pipeline(memory, layout.handle(), color_format)?;
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
        let sampler = DeviceObject::new(memory, unsafe { device.create_sampler(&sampler_info, None) }?);
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(frames_in_flight),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(frames_in_flight),
        ];
        let descriptor_pool = DeviceObject::new(memory, unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(frames_in_flight)
                    .pool_sizes(&pool_sizes),
                None,
            )
        }?);

        Ok(InterfacePipeline {
            descriptor_pool,
            sampler,
            pipeline,
            layout,
            set_layout,
        })
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline.handle()
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout.handle()
    }

    pub fn sampler(&self) -> vk::Sampler {
        self.sampler.handle()
    }

    pub fn allocate_sets(&self, device: &ash::Device, count: usize) -> anyhow::Result<Vec<vk::DescriptorSet>> {
        let layouts = vec![self.set_layout.handle(); count];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool.handle())
            .set_layouts(&layouts);

        Ok(unsafe { device.allocate_descriptor_sets(&info) }?)
    }

    fn create_pipeline(
        memory: &MemoryContext,
        layout: vk::PipelineLayout,
        color_format: vk::Format,
    ) -> anyhow::Result<DeviceObject<vk::Pipeline>> {
        let device = memory.device();
        let code = ash::util::read_spv(&mut Cursor::new(SHADER))?;
        let module = DeviceObject::new(memory, unsafe {
            device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&code), None)
        }?);
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(module.handle())
                .name(c"vertexMain"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(module.handle())
                .name(c"fragmentMain"),
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly =
            vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .line_width(1.0);
        let multisample =
            vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        // Premultiplied alpha: the shader outputs color already multiplied by its alpha.
        let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)];
        let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default();
        let color_formats = [color_format];
        let mut rendering = vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&color_formats);
        let info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .push_next(&mut rendering);
        let pipelines = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None) }
            .map_err(|(_, error)| error)?;

        Ok(DeviceObject::new(memory, pipelines[0]))
    }
}
