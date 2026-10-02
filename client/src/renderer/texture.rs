use std::path::Path;

use anyhow::{Context, bail, ensure};
use ash::vk;

use crate::vulkan::{Allocator, Buffer, Device, Image, ImageDescription, MemoryLocation};

const DDS_MAGIC: &[u8; 4] = b"DDS ";
const DDS_HEADER_SIZE: usize = 148;
const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28;
const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29;
const DXGI_FORMAT_BC4_UNORM: u32 = 80;
const DXGI_FORMAT_BC5_UNORM: u32 = 83;
const DXGI_FORMAT_BC7_UNORM: u32 = 98;
const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99;
const DDS_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;
const CUBE_FACE_COUNT: usize = 6;

#[derive(Clone, Copy)]
pub struct CubeMapHandle(u32);

impl CubeMapHandle {
    pub(super) fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct TextureHandle {
    index: u32,
}

impl TextureHandle {
    pub(super) fn new(index: u32) -> Self {
        Self { index }
    }

    pub fn index(self) -> u32 {
        self.index
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
    Linear,
}

struct DdsImage {
    format: vk::Format,
    width: u32,
    height: u32,
    mip_levels: u32,
    layers: Vec<Vec<u8>>,
}

pub fn upload_cube_map(device: &Device, allocator: &mut Allocator, name: &str, path: &Path) -> anyhow::Result<Image> {
    let cube_map = read_dds(path, ColorSpace::Srgb)?;
    ensure!(
        cube_map.layers.len() == CUBE_FACE_COUNT,
        "{name}: a cube map needs {CUBE_FACE_COUNT} faces"
    );
    ensure!(
        cube_map.width == cube_map.height,
        "{name}: cube map faces must be square"
    );
    upload(device, allocator, name, &cube_map, true)
}

pub fn upload_texture(
    device: &Device,
    allocator: &mut Allocator,
    name: &str,
    path: &Path,
    color_space: ColorSpace,
) -> anyhow::Result<Image> {
    let texture = read_dds(path, color_space)?;
    ensure!(
        texture.layers.len() == 1,
        "{name}: expected a 2D texture, found a cube map"
    );
    upload(device, allocator, name, &texture, false)
}

fn upload(device: &Device, allocator: &mut Allocator, name: &str, dds: &DdsImage, cube: bool) -> anyhow::Result<Image> {
    let mut image = Image::new(
        device,
        allocator,
        &ImageDescription {
            name,
            extent: vk::Extent2D {
                width: dds.width,
                height: dds.height,
            },
            format: dds.format,
            usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            aspect: vk::ImageAspectFlags::COLOR,
            mip_levels: dds.mip_levels,
            cube,
        },
    )?;
    for (layer, data) in dds.layers.iter().enumerate() {
        if let Err(err) = upload_layer(device, allocator, &image, dds, layer as u32, data) {
            unsafe { image.destroy(device, allocator) };
            return Err(err.context(format!("{name}: uploading layer {layer}")));
        }
    }
    Ok(image)
}

fn read_dds(path: &Path, color_space: ColorSpace) -> anyhow::Result<DdsImage> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    ensure!(
        bytes.len() >= DDS_HEADER_SIZE && &bytes[..4] == DDS_MAGIC,
        "{}: not a DDS file",
        path.display()
    );
    let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    ensure!(
        &bytes[84..88] == b"DX10",
        "{}: only DDS files with a DX10 header are supported",
        path.display()
    );

    let height = word(12);
    let width = word(16);
    let mip_levels = word(28).max(1);
    let srgb = color_space == ColorSpace::Srgb;
    let format = match word(128) {
        DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB if srgb => vk::Format::R8G8B8A8_SRGB,
        DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => vk::Format::R8G8B8A8_UNORM,
        DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB if srgb => vk::Format::BC7_SRGB_BLOCK,
        DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB => vk::Format::BC7_UNORM_BLOCK,
        DXGI_FORMAT_BC4_UNORM if !srgb => vk::Format::BC4_UNORM_BLOCK,
        DXGI_FORMAT_BC5_UNORM if !srgb => vk::Format::BC5_UNORM_BLOCK,
        DXGI_FORMAT_BC4_UNORM | DXGI_FORMAT_BC5_UNORM => {
            bail!("{}: BC4 and BC5 hold data, not colors", path.display())
        }
        other => bail!("{}: unsupported DXGI format {other}", path.display()),
    };
    let layer_count = if word(136) & DDS_RESOURCE_MISC_TEXTURECUBE != 0 {
        CUBE_FACE_COUNT
    } else {
        1
    };

    let layer_size: usize = (0..mip_levels)
        .map(|level| mip_size(format, width >> level, height >> level))
        .sum();
    let data = &bytes[DDS_HEADER_SIZE..];
    ensure!(
        data.len() >= layer_size * layer_count,
        "{}: file is truncated",
        path.display()
    );
    let layers = data
        .chunks_exact(layer_size)
        .take(layer_count)
        .map(<[u8]>::to_vec)
        .collect();
    Ok(DdsImage {
        format,
        width,
        height,
        mip_levels,
        layers,
    })
}

fn mip_size(format: vk::Format, width: u32, height: u32) -> usize {
    let width = width.max(1) as usize;
    let height = height.max(1) as usize;
    let blocks = width.div_ceil(4) * height.div_ceil(4);
    match format {
        vk::Format::BC4_UNORM_BLOCK => blocks * 8,
        vk::Format::BC5_UNORM_BLOCK | vk::Format::BC7_UNORM_BLOCK | vk::Format::BC7_SRGB_BLOCK => blocks * 16,
        _ => width * height * 4,
    }
}

fn upload_layer(
    device: &Device,
    allocator: &mut Allocator,
    image: &Image,
    dds: &DdsImage,
    layer: u32,
    data: &[u8],
) -> anyhow::Result<()> {
    let mut staging = Buffer::new(
        device,
        allocator,
        "texture staging",
        data.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    let result = staging
        .write(data)
        .and_then(|()| copy_to_layer(device, &staging, image, dds, layer));
    unsafe { staging.destroy(device, allocator) };
    result
}

fn copy_to_layer(device: &Device, staging: &Buffer, image: &Image, dds: &DdsImage, layer: u32) -> anyhow::Result<()> {
    let handle = device.handle();
    let pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(device.queue_family());
    let pool = unsafe { handle.create_command_pool(&pool_info, None) }?;
    let result = (|| {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { handle.allocate_command_buffers(&allocate_info) }?[0];
        let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        let range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(dds.mip_levels)
            .base_array_layer(layer)
            .layer_count(1);

        let mut offset = 0;
        let regions: Vec<vk::BufferImageCopy> = (0..dds.mip_levels)
            .map(|level| {
                let width = (dds.width >> level).max(1);
                let height = (dds.height >> level).max(1);
                let region = vk::BufferImageCopy::default()
                    .buffer_offset(offset as u64)
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(level)
                            .base_array_layer(layer)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width,
                        height,
                        depth: 1,
                    });
                offset += mip_size(dds.format, width, height);
                region
            })
            .collect();

        unsafe {
            handle.begin_command_buffer(command_buffer, &begin_info)?;
            let to_transfer = [vk::ImageMemoryBarrier2::default()
                .dst_stage_mask(vk::PipelineStageFlags2::COPY)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .image(image.handle())
                .subresource_range(range)];
            handle.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_transfer),
            );
            handle.cmd_copy_buffer_to_image(
                command_buffer,
                staging.handle(),
                image.handle(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
            let to_shader = [vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::COPY)
                .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
                .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(image.handle())
                .subresource_range(range)];
            handle.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_shader),
            );
            handle.end_command_buffer(command_buffer)?;

            let command_buffers = [vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer)];
            let submit = vk::SubmitInfo2::default().command_buffer_infos(&command_buffers);
            handle.queue_submit2(device.queue(), &[submit], vk::Fence::null())?;
            handle.queue_wait_idle(device.queue())?;
        }
        Ok(())
    })();
    unsafe { handle.destroy_command_pool(pool, None) };
    result
}
