use std::io::Cursor;

use ash::vk;

use crate::push_constants::PushConstants;

const SHADER: &[u8] = include_bytes!("../shaders/xui.spv");

pub(crate) struct InterfacePipeline {
    pub set_layout: vk::DescriptorSetLayout,
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub sampler: vk::Sampler,
    pub descriptor_pool: vk::DescriptorPool,
}

impl InterfacePipeline {
    pub unsafe fn new(
        device: &ash::Device,
        color_format: vk::Format,
        frames_in_flight: u32,
    ) -> anyhow::Result<InterfacePipeline> {
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
        let set_layout = unsafe {
            device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None)
        }?;
        let set_layouts = [set_layout];
        let push_constant_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .size(size_of::<PushConstants>() as u32)];
        let layout = unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&set_layouts)
                    .push_constant_ranges(&push_constant_ranges),
                None,
            )
        }?;
        let pipeline = unsafe { InterfacePipeline::create_pipeline(device, layout, color_format) }?;
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
        let sampler = unsafe { device.create_sampler(&sampler_info, None) }?;
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(frames_in_flight),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(frames_in_flight),
        ];
        let descriptor_pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(frames_in_flight)
                    .pool_sizes(&pool_sizes),
                None,
            )
        }?;

        Ok(InterfacePipeline {
            set_layout,
            layout,
            pipeline,
            sampler,
            descriptor_pool,
        })
    }

    pub unsafe fn allocate_sets(&self, device: &ash::Device, count: usize) -> anyhow::Result<Vec<vk::DescriptorSet>> {
        let layouts = vec![self.set_layout; count];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(&layouts);

        Ok(unsafe { device.allocate_descriptor_sets(&info) }?)
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_sampler(self.sampler, None);
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
            device.destroy_descriptor_set_layout(self.set_layout, None);
        }
    }

    unsafe fn create_pipeline(
        device: &ash::Device,
        layout: vk::PipelineLayout,
        color_format: vk::Format,
    ) -> anyhow::Result<vk::Pipeline> {
        let code = ash::util::read_spv(&mut Cursor::new(SHADER))?;
        let module = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&code), None) }?;
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(module)
                .name(c"vertexMain"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(module)
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
        let pipeline = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None) };
        unsafe { device.destroy_shader_module(module, None) };

        Ok(pipeline.map_err(|(_, error)| error)?[0])
    }
}
