use std::path::Path;

use anyhow::{Context, bail, ensure};
use ash::vk;

use crate::vulkan::{Allocator, Buffer, Device, Image, ImageDescription, MemoryLocation};

const DDS_MAGIC: &[u8; 4] = b"DDS ";
const DDS_HEADER_SIZE: usize = 148;
const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28;
const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29;
const DXGI_FORMAT_BC7_UNORM: u32 = 98;
const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99;
const DDS_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;

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

struct DdsImage {
    format: vk::Format,
    size: u32,
    mip_levels: u32,
    faces: Vec<Vec<u8>>,
}

pub fn upload_cube_map(
    device: &Device,
    allocator: &mut Allocator,
    name: &str,
    path: &Path,
) -> anyhow::Result<Image> {
    let cube_map = read_dds(path)?;
    ensure!(cube_map.faces.len() == 6, "{name}: a cube map needs 6 faces");

    let image = Image::new(
        device,
        allocator,
        &ImageDescription {
            name,
            extent: vk::Extent2D {
                width: cube_map.size,
                height: cube_map.size,
            },
            format: cube_map.format,
            usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            aspect: vk::ImageAspectFlags::COLOR,
            mip_levels: cube_map.mip_levels,
            cube: true,
        },
    )?;
    for (face, data) in cube_map.faces.iter().enumerate() {
        upload_face(device, allocator, &image, &cube_map, face as u32, data)
            .with_context(|| format!("{name}: uploading face {face}"))?;
    }
    Ok(image)
}

fn read_dds(path: &Path) -> anyhow::Result<DdsImage> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    ensure!(bytes.len() >= DDS_HEADER_SIZE && &bytes[..4] == DDS_MAGIC, "{}: not a DDS file", path.display());
    let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    ensure!(&bytes[84..88] == b"DX10", "{}: only DDS files with a DX10 header are supported", path.display());

    let (height, width, mip_levels) = (word(12), word(16), word(28).max(1));
    ensure!(width == height, "{}: cube map faces must be square", path.display());
    let format = match word(128) {
        DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => vk::Format::R8G8B8A8_SRGB,
        DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB => vk::Format::BC7_SRGB_BLOCK,
        other => bail!("{}: unsupported DXGI format {other}", path.display()),
    };
    let face_count = if word(136) & DDS_RESOURCE_MISC_TEXTURECUBE != 0 { 6 } else { 1 };

    let face_size: usize = (0..mip_levels).map(|level| mip_size(format, width >> level)).sum();
    let data = &bytes[DDS_HEADER_SIZE..];
    ensure!(data.len() >= face_size * face_count, "{}: file is truncated", path.display());
    let faces = data.chunks_exact(face_size).take(face_count).map(<[u8]>::to_vec).collect();
    Ok(DdsImage {
        format,
        size: width,
        mip_levels,
        faces,
    })
}

fn mip_size(format: vk::Format, size: u32) -> usize {
    let size = size.max(1) as usize;
    match format {
        vk::Format::BC7_SRGB_BLOCK => size.div_ceil(4).pow(2) * 16,
        _ => size * size * 4,
    }
}

fn upload_face(
    device: &Device,
    allocator: &mut Allocator,
    image: &Image,
    cube_map: &DdsImage,
    face: u32,
    data: &[u8],
) -> anyhow::Result<()> {
    let mut staging = Buffer::new(
        device,
        allocator,
        "cube map staging",
        data.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    let result = staging.write(data).and_then(|()| copy_to_face(device, &staging, image, cube_map, face));
    unsafe { staging.destroy(device, allocator) };
    result
}

fn copy_to_face(device: &Device, staging: &Buffer, image: &Image, cube_map: &DdsImage, face: u32) -> anyhow::Result<()> {
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
            .level_count(cube_map.mip_levels)
            .base_array_layer(face)
            .layer_count(1);

        let mut offset = 0;
        let regions: Vec<vk::BufferImageCopy> = (0..cube_map.mip_levels)
            .map(|level| {
                let size = (cube_map.size >> level).max(1);
                let region = vk::BufferImageCopy::default()
                    .buffer_offset(offset as u64)
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(level)
                            .base_array_layer(face)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width: size,
                        height: size,
                        depth: 1,
                    });
                offset += mip_size(cube_map.format, size);
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
            handle.cmd_pipeline_barrier2(command_buffer, &vk::DependencyInfo::default().image_memory_barriers(&to_transfer));
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
            handle.cmd_pipeline_barrier2(command_buffer, &vk::DependencyInfo::default().image_memory_barriers(&to_shader));
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
