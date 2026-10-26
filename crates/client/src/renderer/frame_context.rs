use std::cell::Cell;

use ash::vk;
use render_graph_vulkan::{ImageHandle, PassContext};

use super::command_recorder::CommandRecorder;
use super::frame::Frame;
use super::gpu_data::{MeshDrawData, PushConstants, TerrainDrawData};
use super::graph_textures::GraphTextures;
use super::lens_flare::LensFlare;
use super::material::MaterialBuffer;
use super::scatter::ScatterFrame;
use super::temporal_history::TemporalHistory;
use super::terrain_draw::TerrainDraw;
use super::video_output::VideoOutput;
use super::{MaterialHandle, Scene};
use crate::config::Config;
use crate::vulkan::GraphicsPipeline;

pub(super) const NO_ADDRESS: vk::DeviceAddress = 0;

pub(super) struct FrameContext<'a> {
    pub recorder: CommandRecorder<'a>,
    pub frame: &'a Frame,
    pub frame_slot: usize,
    pub textures: &'a GraphTextures,
    pub temporal: &'a TemporalHistory,
    pub config: &'a Config,
    pub scene: &'a Scene,
    pub materials: &'a [MaterialBuffer],
    pub lens: Option<&'a LensFlare>,
    pub focal_length_pixels: f32,
    pub terrain_draws: &'a [TerrainDraw],
    pub scatter: &'a ScatterFrame,
    pub descriptor_set: vk::DescriptorSet,
    pub video: Option<&'a VideoOutput>,
    pub terrain_triangles: Cell<u64>,
}

impl FrameContext<'_> {
    // `draw` is the address of the draw's record, NO_ADDRESS for a family without one.
    pub fn push_draw_constants(&self, pipeline: &GraphicsPipeline, material: MaterialHandle, draw: vk::DeviceAddress) {
        self.recorder.push_graphics_constants(
            pipeline,
            &PushConstants {
                frame: self.frame.frame_data.device_address(),
                material: self
                    .material(material)
                    .device_address()
                    .expect("only drawable materials are drawn"),
                draw,
            },
        );
    }

    // The record of the frame's terrain draw at `index`.
    pub fn terrain_draw(&self, index: usize) -> vk::DeviceAddress {
        self.frame.terrain_draws.device_address() + (index * size_of::<TerrainDrawData>()) as vk::DeviceAddress
    }

    // The record of the frame's mesh draw at `index`.
    pub fn mesh_draw(&self, index: usize) -> vk::DeviceAddress {
        self.frame.mesh_draws.device_address() + (index * size_of::<MeshDrawData>()) as vk::DeviceAddress
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
