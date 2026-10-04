use std::cell::Cell;

use ash::vk;
use xui::DrawList;

use super::Scene;
use super::command_recorder::CommandRecorder;
use super::frame::Frame;
use super::gpu_data::PushConstants;
use super::render_targets::RenderTargets;
use crate::config::{DebugConfig, RenderConfig};
use crate::vulkan::{Buffer, GraphicsPipeline};

pub(super) struct FrameContext<'a> {
    pub recorder: CommandRecorder<'a>,
    pub frame: &'a Frame,
    pub frame_slot: usize,
    pub targets: &'a RenderTargets,
    pub render: &'a RenderConfig,
    pub debug: &'a DebugConfig,
    pub scene: &'a Scene,
    pub descriptor_set: vk::DescriptorSet,
    pub output_image: vk::Image,
    pub output_view: vk::ImageView,
    pub output_extent: vk::Extent2D,
    pub capture: Option<&'a Buffer>,
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
}
