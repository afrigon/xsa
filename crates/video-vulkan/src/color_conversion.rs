use ash::vk;
use memory_vulkan::{DeviceObject, Image, ImageDescription, ImageKind, ImageMemory, MemoryContext, QueueSharing};

use crate::VideoError;
use crate::conversion_constants::ConversionConstants;
use crate::conversion_source::ConversionSource;

const SHADER: &[u8] = include_bytes!("../shaders/color_conversion.spv");
const ENTRY_POINT: &std::ffi::CStr = c"computeMain";
const WORKGROUP_SIZE: u32 = 8;
// Each invocation converts a 2×2 block.
const BLOCK_SIZE: u32 = 2;

// The work a barrier waits for or holds back: pipeline stages and the memory accesses they make.
#[derive(Clone, Copy)]
struct Scope {
    stage: vk::PipelineStageFlags2,
    access: vk::AccessFlags2,
}

/// Converts rendered frames into a `VideoEncoder`'s NV12 inputs on the host's command buffer: a compute pass writes
/// the luma and chroma planes to images of their own, which are then copied into the input's planes.
pub struct ColorConversion {
    memory: MemoryContext,
    extent: vk::Extent2D,
    luma: Image,
    chroma: Image,
    descriptor_sets: Vec<vk::DescriptorSet>,
    _descriptor_pool: DeviceObject<vk::DescriptorPool>,
    pipeline: DeviceObject<vk::Pipeline>,
    pipeline_layout: DeviceObject<vk::PipelineLayout>,
    _set_layout: DeviceObject<vk::DescriptorSetLayout>,
}

impl ColorConversion {
    /// A conversion into inputs of `coded_extent`, recorded by up to `frames_in_flight` frames at once.
    pub fn new(
        memory: &MemoryContext,
        coded_extent: vk::Extent2D,
        frames_in_flight: usize,
    ) -> Result<ColorConversion, VideoError> {
        let device = memory.device();
        let plane = |name, format, extent| {
            Image::new(
                memory,
                &ImageDescription {
                    name,
                    extent,
                    format,
                    usage: vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
                    aspect: vk::ImageAspectFlags::COLOR,
                    mip_levels: 1,
                    kind: ImageKind::Single,
                    memory: ImageMemory::Dedicated,
                    sharing: QueueSharing::Exclusive,
                },
            )
        };
        let luma = plane("video luma", vk::Format::R8_UNORM, coded_extent)?;
        let chroma = plane(
            "video chroma",
            vk::Format::R8G8_UNORM,
            vk::Extent2D {
                width: coded_extent.width / BLOCK_SIZE,
                height: coded_extent.height / BLOCK_SIZE,
            },
        )?;
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let set_layout = DeviceObject::new(memory, unsafe {
            device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None)
        }?);
        let push_constants = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .size(size_of::<ConversionConstants>() as u32)];
        let set_layouts = [set_layout.handle()];
        let pipeline_layout = DeviceObject::new(memory, unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&set_layouts)
                    .push_constant_ranges(&push_constants),
                None,
            )
        }?);
        let pipeline = ColorConversion::create_pipeline(memory, pipeline_layout.handle())?;
        let count = frames_in_flight as u32;
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(count),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(2 * count),
        ];
        let descriptor_pool = DeviceObject::new(memory, unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(count)
                    .pool_sizes(&pool_sizes),
                None,
            )
        }?);
        let layouts = vec![set_layout.handle(); frames_in_flight];
        let descriptor_sets = unsafe {
            device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool.handle())
                    .set_layouts(&layouts),
            )
        }?;

        Ok(ColorConversion {
            memory: memory.clone(),
            extent: coded_extent,
            luma,
            chroma,
            descriptor_sets,
            _descriptor_pool: descriptor_pool,
            pipeline,
            pipeline_layout,
            _set_layout: set_layout,
        })
    }

    /// Converts `source` into `target`, a `VideoEncoder` input, leaving it in `VIDEO_ENCODE_SRC_KHR`. `frame` picks
    /// the descriptor set: no other recording of the same frame may still be pending on the GPU.
    pub fn record(&self, command_buffer: vk::CommandBuffer, frame: usize, source: &ConversionSource, target: &Image) {
        let device = self.memory.device();
        let set = self.descriptor_sets[frame];
        let source_info = [vk::DescriptorImageInfo::default()
            .image_view(source.view)
            .image_layout(source.layout)];
        let luma_info = [vk::DescriptorImageInfo::default()
            .image_view(self.luma.view())
            .image_layout(vk::ImageLayout::GENERAL)];
        let chroma_info = [vk::DescriptorImageInfo::default()
            .image_view(self.chroma.view())
            .image_layout(vk::ImageLayout::GENERAL)];
        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .image_info(&source_info),
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(&luma_info),
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(&chroma_info),
        ];
        unsafe { device.update_descriptor_sets(&writes, &[]) };

        // The previous frame's copies read the planes; this frame overwrites them whole.
        let planes_writable = [&self.luma, &self.chroma].map(|plane| {
            ColorConversion::barrier(
                plane,
                Scope {
                    stage: vk::PipelineStageFlags2::COPY,
                    access: vk::AccessFlags2::NONE,
                },
                Scope {
                    stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
                    access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
                },
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::GENERAL,
            )
        });
        let constants = ConversionConstants {
            source_extent: [source.extent.width as i32, source.extent.height as i32],
            encode_transfer: u32::from(source.loads_linear()),
        };
        let extent = self.extent;
        let groups = |size: u32| size.div_ceil(BLOCK_SIZE * WORKGROUP_SIZE);

        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&planes_writable),
            );
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, self.pipeline.handle());
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout.handle(),
                0,
                &[set],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout.handle(),
                vk::ShaderStageFlags::COMPUTE,
                0,
                constants.bytes(),
            );
            device.cmd_dispatch(command_buffer, groups(extent.width), groups(extent.height), 1);
        }

        self.copy_planes(command_buffer, target);
    }

    fn copy_planes(&self, command_buffer: vk::CommandBuffer, target: &Image) {
        let device = self.memory.device();
        let written = Scope {
            stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
        };
        let copied = Scope {
            stage: vk::PipelineStageFlags2::COPY,
            access: vk::AccessFlags2::TRANSFER_READ,
        };
        // The encoder finished reading the input before handing it out again; its contents are replaced whole.
        let before = [
            ColorConversion::barrier(
                &self.luma,
                written,
                copied,
                vk::ImageLayout::GENERAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ),
            ColorConversion::barrier(
                &self.chroma,
                written,
                copied,
                vk::ImageLayout::GENERAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ),
            ColorConversion::barrier(
                target,
                Scope {
                    stage: vk::PipelineStageFlags2::NONE,
                    access: vk::AccessFlags2::NONE,
                },
                Scope {
                    stage: vk::PipelineStageFlags2::COPY,
                    access: vk::AccessFlags2::TRANSFER_WRITE,
                },
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ),
        ];
        let extent = self.extent;
        let copy = |aspect, extent: vk::Extent2D| {
            vk::ImageCopy::default()
                .src_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1),
                )
                .dst_subresource(vk::ImageSubresourceLayers::default().aspect_mask(aspect).layer_count(1))
                .extent(vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                })
        };
        let chroma_extent = vk::Extent2D {
            width: extent.width / BLOCK_SIZE,
            height: extent.height / BLOCK_SIZE,
        };
        // The semaphore the encode queue waits on makes the copies visible to it.
        let after = [ColorConversion::barrier(
            target,
            Scope {
                stage: vk::PipelineStageFlags2::COPY,
                access: vk::AccessFlags2::TRANSFER_WRITE,
            },
            Scope {
                stage: vk::PipelineStageFlags2::NONE,
                access: vk::AccessFlags2::NONE,
            },
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::VIDEO_ENCODE_SRC_KHR,
        )];

        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&before),
            );
            device.cmd_copy_image(
                command_buffer,
                self.luma.handle(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                target.handle(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy(vk::ImageAspectFlags::PLANE_0, extent)],
            );
            device.cmd_copy_image(
                command_buffer,
                self.chroma.handle(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                target.handle(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy(vk::ImageAspectFlags::PLANE_1, chroma_extent)],
            );
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&after),
            );
        }
    }

    fn barrier(
        image: &Image,
        source: Scope,
        destination: Scope,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> vk::ImageMemoryBarrier2<'static> {
        vk::ImageMemoryBarrier2::default()
            .src_stage_mask(source.stage)
            .src_access_mask(source.access)
            .dst_stage_mask(destination.stage)
            .dst_access_mask(destination.access)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(image.handle())
            .subresource_range(image.subresource_range())
    }

    fn create_pipeline(
        memory: &MemoryContext,
        layout: vk::PipelineLayout,
    ) -> Result<DeviceObject<vk::Pipeline>, VideoError> {
        let device = memory.device();
        let code: Vec<u32> = SHADER
            .as_chunks::<4>()
            .0
            .iter()
            .map(|word| u32::from_le_bytes(*word))
            .collect();
        let module = DeviceObject::new(memory, unsafe {
            device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&code), None)
        }?);
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(module.handle())
            .name(ENTRY_POINT);
        let info = vk::ComputePipelineCreateInfo::default().stage(stage).layout(layout);
        let pipelines = unsafe { device.create_compute_pipelines(vk::PipelineCache::null(), &[info], None) }
            .map_err(|(_, result)| result)?;

        Ok(DeviceObject::new(memory, pipelines[0]))
    }
}
