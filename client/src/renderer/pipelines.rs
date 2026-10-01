use std::mem::offset_of;

use ash::vk;

use super::DEPTH_FORMAT;
use super::gpu_data::PushConstants;
use super::material::Shader;
use crate::mesh::Vertex;
use crate::vulkan::{BindlessTextures, Device, GraphicsPipeline, GraphicsPipelineDescription};

const POINT_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/point.spv"));

pub(super) struct Pipelines {
    shaders: Vec<GraphicsPipeline>,
    point: GraphicsPipeline,
}

impl Pipelines {
    pub fn new(device: &Device, bindless: &BindlessTextures, color_format: vk::Format) -> anyhow::Result<Self> {
        let shaders = Shader::ALL
            .iter()
            .map(|shader| create_shader_pipeline(device, bindless, *shader, color_format))
            .collect::<anyhow::Result<_>>()?;
        let point = GraphicsPipeline::new(
            device,
            &GraphicsPipelineDescription {
                spirv: POINT_SHADER,
                color_format,
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
        Ok(Self { shaders, point })
    }

    pub fn shader(&self, shader: Shader) -> &GraphicsPipeline {
        &self.shaders[shader.index()]
    }

    pub fn point(&self) -> &GraphicsPipeline {
        &self.point
    }

    pub unsafe fn destroy(&mut self, device: &Device) {
        unsafe {
            for pipeline in &mut self.shaders {
                pipeline.destroy(device);
            }
            self.point.destroy(device);
        }
    }
}

fn create_shader_pipeline(
    device: &Device,
    bindless: &BindlessTextures,
    shader: Shader,
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
        spirv: shader.spirv(),
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
