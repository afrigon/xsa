use std::cell::Cell;

use ash::vk;
use render_graph_vulkan::{ImageHandle, PassContext};
use xui::DrawList;

use super::command_recorder::CommandRecorder;
use super::frame::Frame;
use super::gpu_data::PushConstants;
use super::graph_textures::GraphTextures;
use super::lens_flare::LensFlare;
use super::material::MaterialBuffer;
use super::temporal_history::TemporalHistory;
use super::terrain_draw::TerrainDraw;
use super::{MaterialHandle, Scene};
use crate::config::RenderConfig;
use crate::vulkan::GraphicsPipeline;

const NULL_ADDRESS: vk::DeviceAddress = 0;

pub(super) struct FrameContext<'a> {
    pub recorder: CommandRecorder<'a>,
    pub frame: &'a Frame,
    pub frame_slot: usize,
    pub textures: &'a GraphTextures,
    pub temporal: &'a TemporalHistory,
    pub render: &'a RenderConfig,
    pub scene: &'a Scene,
    pub materials: &'a [MaterialBuffer],
    pub lens: Option<&'a LensFlare>,
    pub focal_length_pixels: f32,
    pub terrain_draws: &'a [TerrainDraw],
    pub descriptor_set: vk::DescriptorSet,
    pub user_interface: &'a DrawList,
    pub triangles: Cell<u64>,
}

impl FrameContext<'_> {
    pub fn push_draw_constants(
        &self,
        pipeline: &GraphicsPipeline,
        object_index: usize,
        material: MaterialHandle,
        first_tile: u32,
        terrain_data: Option<vk::DeviceAddress>,
    ) {
        self.recorder.push_graphics_constants(
            pipeline,
            &PushConstants {
                frame: self.frame.frame_data.device_address(),
                objects: self.frame.objects.device_address(),
                material: self
                    .material(material)
                    .device_address()
                    .expect("only drawable materials are drawn"),
                object_index: object_index as u32,
                first_tile,
                tiles: self.frame.tiles.device_address(),
                terrain: terrain_data.unwrap_or(NULL_ADDRESS),
            },
        );
    }

    // Whether its parameters are in GPU memory.
    pub fn is_drawable(&self, material: MaterialHandle) -> bool {
        self.material(material).device_address().is_some()
    }

    pub fn material(&self, handle: MaterialHandle) -> &MaterialBuffer {
        &self.materials[handle.index]
    }

    pub fn texture(&self, pass: &PassContext, image: ImageHandle) -> u32 {
        self.textures.texture(pass.image_id(image))
    }

    pub fn storage_image(&self, pass: &PassContext, image: ImageHandle, level: u32) -> u32 {
        self.textures.storage_image(pass.image_id(image), level)
    }
}
