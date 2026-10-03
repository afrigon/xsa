use std::io::Cursor;

use anyhow::Context;
use ash::vk;

use super::GraphicsPipelineDescription;
use crate::vulkan::Device;

pub struct GraphicsPipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl GraphicsPipeline {
    pub fn new(device: &Device, description: &GraphicsPipelineDescription) -> anyhow::Result<Self> {
        let code = ash::util::read_spv(&mut Cursor::new(description.spirv)).context("reading SPIR-V")?;
        let module_info = vk::ShaderModuleCreateInfo::default().code(&code);
        let module = unsafe { device.handle().create_shader_module(&module_info, None) }?;
        let dynamic_polygon_mode = device.extended_dynamic_state3().is_some();
        let pipeline = GraphicsPipeline::create_pipeline(device.handle(), module, description, dynamic_polygon_mode);
        unsafe { device.handle().destroy_shader_module(module, None) };
        pipeline
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub unsafe fn destroy(&mut self, device: &Device) {
        let device = device.handle();
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
        }
    }

    fn create_pipeline(
        device: &ash::Device,
        module: vk::ShaderModule,
        description: &GraphicsPipelineDescription,
        dynamic_polygon_mode: bool,
    ) -> anyhow::Result<GraphicsPipeline> {
        let push_constant_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .size(description.push_constant_size)];
        let mut layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(description.descriptor_set_layouts);
        if description.push_constant_size > 0 {
            layout_info = layout_info.push_constant_ranges(&push_constant_ranges);
        }
        let layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;

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
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(description.vertex_bindings)
            .vertex_attribute_descriptions(description.vertex_attributes);
        let input_assembly =
            vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(description.cull_mode)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);
        let multisample =
            vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(description.additive_blend)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE)
            .alpha_blend_op(vk::BlendOp::ADD)];
        let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let mut dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        if dynamic_polygon_mode {
            dynamic_states.push(vk::DynamicState::POLYGON_MODE_EXT);
        }
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(description.depth_format.is_some())
            .depth_write_enable(description.depth_format.is_some() && description.depth_write)
            .depth_compare_op(description.depth_compare_op);
        let color_formats = [description.color_format];
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_formats)
            .depth_attachment_format(description.depth_format.unwrap_or(vk::Format::UNDEFINED));

        let create_info = vk::GraphicsPipelineCreateInfo::default()
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
        let pipelines = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None) };
        match pipelines {
            Ok(pipelines) => Ok(GraphicsPipeline {
                pipeline: pipelines[0],
                layout,
            }),
            Err((_, err)) => {
                unsafe { device.destroy_pipeline_layout(layout, None) };
                Err(anyhow::Error::from(err).context("creating the graphics pipeline"))
            }
        }
    }
}
