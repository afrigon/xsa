use ash::vk;

use super::gpu_context::GpuContext;
use crate::vulkan::{Image, ImageDescription, SAMPLED_LAYOUT};

pub(super) struct RenderTargets {
    pub extent: vk::Extent2D,
    pub depth: Image,
    pub depth_texture: u32,
    pub hdr: Image,
    pub hdr_texture: u32,
    pub motion: Image,
    pub motion_texture: u32,
}

impl RenderTargets {
    pub const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
    pub const HDR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
    pub const MOTION_FORMAT: vk::Format = vk::Format::R16G16_SFLOAT;

    pub fn new(gpu: &mut GpuContext, extent: vk::Extent2D) -> anyhow::Result<RenderTargets> {
        let depth = RenderTargets::create_depth(gpu, extent)?;
        let depth_texture = gpu.bindless.add_texture(&gpu.device, depth.view(), SAMPLED_LAYOUT)?;
        let hdr = RenderTargets::create_hdr(gpu, extent)?;
        let hdr_texture = gpu.bindless.add_texture(&gpu.device, hdr.view(), SAMPLED_LAYOUT)?;
        let motion = RenderTargets::create_motion(gpu, extent)?;
        let motion_texture = gpu.bindless.add_texture(&gpu.device, motion.view(), SAMPLED_LAYOUT)?;

        Ok(RenderTargets {
            extent,
            depth,
            depth_texture,
            hdr,
            hdr_texture,
            motion,
            motion_texture,
        })
    }

    pub fn recreate(&mut self, gpu: &mut GpuContext, extent: vk::Extent2D) -> anyhow::Result<()> {
        let depth = RenderTargets::create_depth(gpu, extent)?;
        gpu.bindless
            .set_texture(&gpu.device, self.depth_texture, depth.view(), SAMPLED_LAYOUT);
        let mut old_depth = std::mem::replace(&mut self.depth, depth);
        unsafe { old_depth.destroy(&gpu.device, &mut gpu.allocator) };

        let hdr = RenderTargets::create_hdr(gpu, extent)?;
        gpu.bindless
            .set_texture(&gpu.device, self.hdr_texture, hdr.view(), SAMPLED_LAYOUT);
        let mut old_hdr = std::mem::replace(&mut self.hdr, hdr);
        unsafe { old_hdr.destroy(&gpu.device, &mut gpu.allocator) };

        let motion = RenderTargets::create_motion(gpu, extent)?;
        gpu.bindless
            .set_texture(&gpu.device, self.motion_texture, motion.view(), SAMPLED_LAYOUT);
        let mut old_motion = std::mem::replace(&mut self.motion, motion);
        unsafe { old_motion.destroy(&gpu.device, &mut gpu.allocator) };

        self.extent = extent;

        Ok(())
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            self.depth.destroy(&gpu.device, &mut gpu.allocator);
            self.hdr.destroy(&gpu.device, &mut gpu.allocator);
            self.motion.destroy(&gpu.device, &mut gpu.allocator);
        }
    }

    fn create_depth(gpu: &mut GpuContext, extent: vk::Extent2D) -> anyhow::Result<Image> {
        Image::new(
            &gpu.device,
            &mut gpu.allocator,
            &ImageDescription {
                name: "depth",
                extent,
                format: RenderTargets::DEPTH_FORMAT,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                aspect: vk::ImageAspectFlags::DEPTH,
                mip_levels: 1,
                cube: false,
            },
        )
    }

    fn create_hdr(gpu: &mut GpuContext, extent: vk::Extent2D) -> anyhow::Result<Image> {
        Image::new(
            &gpu.device,
            &mut gpu.allocator,
            &ImageDescription {
                name: "hdr color",
                extent,
                format: RenderTargets::HDR_FORMAT,
                usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                cube: false,
            },
        )
    }

    fn create_motion(gpu: &mut GpuContext, extent: vk::Extent2D) -> anyhow::Result<Image> {
        Image::new(
            &gpu.device,
            &mut gpu.allocator,
            &ImageDescription {
                name: "motion vectors",
                extent,
                format: RenderTargets::MOTION_FORMAT,
                usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                cube: false,
            },
        )
    }
}
