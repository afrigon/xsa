use ash::vk;

use super::command_recorder::CommandRecorder;
use super::frame::Frame;
use super::gpu_data::PushConstants;
use super::render_targets::RenderTargets;
use super::{RenderSettings, Scene};
use crate::vulkan::GraphicsPipeline;

pub(super) struct FrameContext<'a> {
    pub recorder: CommandRecorder<'a>,
    pub frame: &'a Frame,
    pub targets: &'a RenderTargets,
    pub settings: &'a RenderSettings,
    pub scene: &'a Scene,
    pub descriptor_set: vk::DescriptorSet,
    pub output_image: vk::Image,
    pub output_view: vk::ImageView,
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
