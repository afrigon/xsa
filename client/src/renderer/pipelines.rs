use std::mem::offset_of;

use ash::vk;

use super::DEPTH_FORMAT;
use super::gpu_data::{HistogramPushConstants, PushConstants};
use super::material::Shader;
use crate::mesh::Vertex;
use crate::vulkan::{BindlessTextures, ComputePipeline, Device, GraphicsPipeline, GraphicsPipelineDescription};

pub const POINT_SHADER_PATH: &str = "point";
pub const TONEMAP_SHADER_PATH: &str = "tonemap";
pub const HISTOGRAM_SHADER_PATH: &str = "histogram";

pub struct ShaderBinaries {
    pub shaders: Vec<Vec<u8>>,
    pub point: Vec<u8>,
    pub tonemap: Vec<u8>,
    pub histogram: Vec<u8>,
}

pub(super) struct Pipelines {
    shaders: Vec<GraphicsPipeline>,
    point: GraphicsPipeline,
    tonemap: GraphicsPipeline,
    histogram: ComputePipeline,
}

impl Pipelines {
    pub fn new(
        device: &Device,
        bindless: &BindlessTextures,
        binaries: &ShaderBinaries,
        scene_format: vk::Format,
        swapchain_format: vk::Format,
    ) -> anyhow::Result<Self> {
        let shaders = Shader::ALL
            .iter()
            .map(|shader| {
                create_shader_pipeline(
                    device,
                    bindless,
                    *shader,
                    &binaries.shaders[shader.index()],
                    scene_format,
                )
            })
            .collect::<anyhow::Result<_>>()?;
        let point = GraphicsPipeline::new(
            device,
            &GraphicsPipelineDescription {
                spirv: &binaries.point,
                color_format: scene_format,
                depth_format: Some(DEPTH_FORMAT),
                depth_compare_op: vk::CompareOp::GREATER,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                additive_blend: true,
                vertex_bindings: &[],
                vertex_attributes: &[],
                push_constant_size: size_of::<PushConstants>() as u32,
                descriptor_set_layouts: &[bindless.layout()],
            },
        )?;
        let tonemap = GraphicsPipeline::new(
            device,
            &GraphicsPipelineDescription {
                spirv: &binaries.tonemap,
                color_format: swapchain_format,
                depth_format: None,
                depth_compare_op: vk::CompareOp::ALWAYS,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                additive_blend: false,
                vertex_bindings: &[],
                vertex_attributes: &[],
                push_constant_size: size_of::<PushConstants>() as u32,
                descriptor_set_layouts: &[bindless.layout()],
            },
        )?;
        let histogram = ComputePipeline::new(
            device,
            &binaries.histogram,
            size_of::<HistogramPushConstants>() as u32,
            &[bindless.layout()],
        )?;
        Ok(Self {
            shaders,
            point,
            tonemap,
            histogram,
        })
    }

    pub fn shader(&self, shader: Shader) -> &GraphicsPipeline {
        &self.shaders[shader.index()]
    }

    pub fn point(&self) -> &GraphicsPipeline {
        &self.point
    }

    pub fn tonemap(&self) -> &GraphicsPipeline {
        &self.tonemap
    }

    pub fn histogram(&self) -> &ComputePipeline {
        &self.histogram
    }

    pub unsafe fn destroy(&mut self, device: &Device) {
        unsafe {
            for pipeline in &mut self.shaders {
                pipeline.destroy(device);
            }
            self.point.destroy(device);
            self.tonemap.destroy(device);
            self.histogram.destroy(device);
        }
    }
}

fn create_shader_pipeline(
    device: &Device,
    bindless: &BindlessTextures,
    shader: Shader,
    spirv: &[u8],
    color_format: vk::Format,
) -> anyhow::Result<GraphicsPipeline> {
    let vertex_bindings = [vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(size_of::<Vertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX)];
    let vertex_attributes = [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, position) as u32),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, normal) as u32),
    ];
    let mesh_description = GraphicsPipelineDescription {
        spirv,
        color_format,
        depth_format: Some(DEPTH_FORMAT),
        depth_compare_op: vk::CompareOp::GREATER,
        depth_write: true,
        cull_mode: vk::CullModeFlags::BACK,
        additive_blend: false,
        vertex_bindings: &vertex_bindings,
        vertex_attributes: &vertex_attributes,
        push_constant_size: size_of::<PushConstants>() as u32,
        descriptor_set_layouts: &[bindless.layout()],
    };
    let description = match shader {
        Shader::Skybox => GraphicsPipelineDescription {
            depth_compare_op: vk::CompareOp::GREATER_OR_EQUAL,
            depth_write: false,
            cull_mode: vk::CullModeFlags::NONE,
            vertex_bindings: &[],
            vertex_attributes: &[],
            ..mesh_description
        },
        _ => mesh_description,
    };
    GraphicsPipeline::new(device, &description)
}
