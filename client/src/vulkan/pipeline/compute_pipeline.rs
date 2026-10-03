use std::io::Cursor;

use anyhow::Context;
use ash::vk;

use crate::vulkan::Device;

pub struct ComputePipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl ComputePipeline {
    pub fn new(
        device: &Device,
        spirv: &[u8],
        push_constant_size: u32,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> anyhow::Result<Self> {
        let code = ash::util::read_spv(&mut Cursor::new(spirv)).context("reading SPIR-V")?;
        let device = device.handle();
        let module_info = vk::ShaderModuleCreateInfo::default().code(&code);
        let module = unsafe { device.create_shader_module(&module_info, None) }?;
        let push_constant_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .size(push_constant_size)];
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);
        let layout = match unsafe { device.create_pipeline_layout(&layout_info, None) } {
            Ok(layout) => layout,
            Err(err) => {
                unsafe { device.destroy_shader_module(module, None) };
                return Err(err.into());
            }
        };
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(module)
            .name(c"computeMain");
        let create_info = vk::ComputePipelineCreateInfo::default().stage(stage).layout(layout);
        let pipelines = unsafe { device.create_compute_pipelines(vk::PipelineCache::null(), &[create_info], None) };
        unsafe { device.destroy_shader_module(module, None) };
        match pipelines {
            Ok(pipelines) => Ok(Self {
                pipeline: pipelines[0],
                layout,
            }),
            Err((_, err)) => {
                unsafe { device.destroy_pipeline_layout(layout, None) };
                Err(anyhow::Error::from(err).context("creating the compute pipeline"))
            }
        }
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
}
