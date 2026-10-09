use std::cell::Cell;

use ash::vk;
use render_graph::{ImageHandle, PassContext};
use xui::DrawList;

use super::command_recorder::CommandRecorder;
use super::frame::Frame;
use super::gpu_data::PushConstants;
use super::graph_textures::GraphTextures;
use super::temporal_history::TemporalHistory;
use super::{ObjectHandle, Scene};
use crate::config::{DebugConfig, RenderConfig};
use crate::vulkan::GraphicsPipeline;

pub(super) struct FrameContext<'a> {
    pub recorder: CommandRecorder<'a>,
    pub frame: &'a Frame,
    pub frame_slot: usize,
    pub textures: &'a GraphTextures,
    pub temporal: &'a TemporalHistory,
    pub render: &'a RenderConfig,
    pub debug: &'a DebugConfig,
    pub scene: &'a Scene,
    pub visible_objects: &'a [ObjectHandle],
    pub descriptor_set: vk::DescriptorSet,
    pub user_interface: &'a DrawList,
    pub triangles: Cell<u64>,
}

impl FrameContext<'_> {
    pub fn push_draw_constants(&self, pipeline: &GraphicsPipeline, object_index: usize, material_index: usize) {
        self.recorder.push_graphics_constants(
            pipeline,
            &PushConstants {
                frame: self.frame.frame_data.device_address(),
                objects: self.frame.objects.device_address(),
                materials: self.frame.materials.device_address(),
                object_index: object_index as u32,
                material_index: material_index as u32,
            },
        );
    }

    pub fn texture(&self, pass: &PassContext, image: ImageHandle) -> anyhow::Result<u32> {
        self.textures.texture(pass.image_id(image))
    }

    pub fn storage_image(&self, pass: &PassContext, image: ImageHandle, level: u32) -> anyhow::Result<u32> {
        self.textures.storage_image(pass.image_id(image), level)
    }
}
