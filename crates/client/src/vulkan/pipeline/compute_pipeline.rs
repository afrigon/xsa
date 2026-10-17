use std::io::Cursor;

use anyhow::Context;
use ash::vk;
use memory_vulkan::{DeviceObject, MemoryContext};

pub struct ComputePipeline {
    // Declaration order is destruction order: the pipeline before its layout.
    pipeline: DeviceObject<vk::Pipeline>,
    layout: DeviceObject<vk::PipelineLayout>,
}

impl ComputePipeline {
    pub fn new(
        memory: &MemoryContext,
        spirv: &[u8],
        push_constant_size: u32,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> anyhow::Result<Self> {
        let code = ash::util::read_spv(&mut Cursor::new(spirv)).context("reading SPIR-V")?;
        let device = memory.device();
        let module_info = vk::ShaderModuleCreateInfo::default().code(&code);
        let module = DeviceObject::new(memory, unsafe { device.create_shader_module(&module_info, None) }?);
        let push_constant_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .size(push_constant_size)];
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);
        let layout = DeviceObject::new(memory, unsafe { device.create_pipeline_layout(&layout_info, None) }?);
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(module.handle())
            .name(c"computeMain");
        let create_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage)
            .layout(layout.handle());
        let pipelines = unsafe { device.create_compute_pipelines(vk::PipelineCache::null(), &[create_info], None) }
            .map_err(|(_, err)| anyhow::Error::from(err).context("creating the compute pipeline"))?;

        Ok(Self {
            pipeline: DeviceObject::new(memory, pipelines[0]),
            layout,
        })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.pipeline.handle()
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout.handle()
    }
}
