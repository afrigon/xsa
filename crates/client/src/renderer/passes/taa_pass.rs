use ash::vk;

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::TaaPushConstants;
use crate::renderer::render_pass::RenderPass;
use crate::renderer::render_targets::RenderTargets;
use crate::renderer::temporal_history::HISTORY_IMAGE_COUNT;
use crate::vulkan::{ComputePipeline, Image, ImageDescription};

const HISTORY_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
// Must match the workgroup size in taa.slang.
const WORKGROUP_SIZE: u32 = 8;

// Temporal anti-aliasing: blends each jittered frame into a history reprojected along the motion vectors. The two
// history images alternate between being read as last frame's result and written as this frame's.
pub(in crate::renderer) struct TaaPass {
    pipeline: ComputePipeline,
    history: Vec<Image>,
    textures: Vec<u32>,
    storage_images: Vec<u32>,
}

impl TaaPass {
    pub fn new(gpu: &mut GpuContext, binaries: &ShaderBinaries, extent: vk::Extent2D) -> anyhow::Result<TaaPass> {
        let pipeline = ComputePipeline::new(
            &gpu.device,
            &binaries.taa,
            size_of::<TaaPushConstants>() as u32,
            &[gpu.bindless.layout()],
        )?;
        let history = TaaPass::create_history(gpu, extent)?;
        let textures = history
            .iter()
            .map(|image| {
                gpu.bindless
                    .add_texture(&gpu.device, image.view(), vk::ImageLayout::GENERAL)
            })
            .collect::<anyhow::Result<_>>()?;
        let storage_images = history
            .iter()
            .map(|image| gpu.bindless.add_storage_image(&gpu.device, image.view()))
            .collect::<anyhow::Result<_>>()?;

        Ok(TaaPass {
            pipeline,
            history,
            textures,
            storage_images,
        })
    }

    pub fn textures(&self) -> Vec<u32> {
        self.textures.clone()
    }

    fn create_history(gpu: &mut GpuContext, extent: vk::Extent2D) -> anyhow::Result<Vec<Image>> {
        (0..HISTORY_IMAGE_COUNT)
            .map(|_| {
                Image::new(
                    &gpu.device,
                    &mut gpu.allocator,
                    &ImageDescription {
                        name: "taa history",
                        extent,
                        format: HISTORY_FORMAT,
                        usage: vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                        aspect: vk::ImageAspectFlags::COLOR,
                        mip_levels: 1,
                        cube: false,
                    },
                )
            })
            .collect()
    }

    unsafe fn destroy_history(gpu: &mut GpuContext, history: &mut [Image]) {
        for image in history {
            unsafe { image.destroy(&gpu.device, &mut gpu.allocator) };
        }
    }
}

impl RenderPass for TaaPass {
    fn record(&mut self, frame: &FrameContext) -> anyhow::Result<()> {
        if !frame.render.taa.enabled {
            return Ok(());
        }

        let recorder = &frame.recorder;
        let temporal = frame.temporal;
        let current = temporal.current();
        let previous = temporal.previous();
        recorder.history_start(self.history[current].handle());

        if temporal.is_valid() {
            recorder.history_previous_to_compute(self.history[previous].handle());
        }

        recorder.bind_compute(&self.pipeline, frame.descriptor_set);
        recorder.push_compute_constants(
            &self.pipeline,
            &TaaPushConstants {
                frame: frame.frame.frame_data.device_address(),
                color_texture: frame.targets.hdr_texture,
                motion_texture: frame.targets.motion_texture,
                depth_texture: frame.targets.depth_texture,
                history_texture: self.textures[previous],
                output_image: self.storage_images[current],
                history_valid: u32::from(temporal.is_valid()),
                history_exposure_scale: temporal.history_exposure_scale(),
                padding: 0,
            },
        );
        recorder.dispatch(frame.targets.extent, WORKGROUP_SIZE);
        recorder.history_to_readers(self.history[current].handle());

        Ok(())
    }

    fn resize(&mut self, gpu: &mut GpuContext, targets: &RenderTargets) -> anyhow::Result<()> {
        let history = TaaPass::create_history(gpu, targets.extent)?;

        for ((texture, storage_image), image) in self.textures.iter().zip(&self.storage_images).zip(&history) {
            gpu.bindless
                .set_texture(&gpu.device, *texture, image.view(), vk::ImageLayout::GENERAL);
            gpu.bindless
                .set_storage_image(&gpu.device, *storage_image, image.view());
        }

        let mut old = std::mem::replace(&mut self.history, history);
        unsafe { TaaPass::destroy_history(gpu, &mut old) };

        Ok(())
    }

    unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            self.pipeline.destroy(&gpu.device);
            TaaPass::destroy_history(gpu, &mut self.history);
        }
    }
}
