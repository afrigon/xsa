//! H.265 video encoding on the GPU's hardware encoder through Vulkan Video.
//!
//! A `ColorConversion` records the conversion of a rendered frame into the encoder's YUV input on the host's command
//! buffer; a `VideoEncoder` encodes it on the encode queue and hands back each frame's bitstream; an `Mp4Writer` wraps
//! the bitstream in an MP4 file. Every GPU object follows `memory-vulkan`'s lifetime rules.

mod color_conversion;
mod conversion_constants;
mod conversion_source;
mod encode_feedback;
mod encode_queue;
mod encode_session;
mod encoded_frame;
mod encoder_capabilities;
mod h265_profile;
mod mp4_box;
mod mp4_writer;
mod nal_units;
mod picture_geometry;
mod video_encoder;
mod video_error;
mod video_session;
mod video_session_parameters;

pub use color_conversion::ColorConversion;
pub use conversion_source::ConversionSource;
pub use encode_queue::EncodeQueue;
pub use encoded_frame::EncodedFrame;
pub use mp4_writer::Mp4Writer;
pub use video_encoder::VideoEncoder;
pub use video_error::VideoError;

#[cfg(test)]
mod tests {
    use ash::{khr, vk};
    use memory_vulkan::{TestDevice, TestRequirements};

    use crate::encode_session::EncodeSession;
    use crate::encoder_capabilities::EncoderCapabilities;
    use crate::picture_geometry::PictureGeometry;
    use crate::{ColorConversion, ConversionSource, EncodeQueue, Mp4Writer, VideoEncoder};

    pub(crate) fn encode_device() -> Option<TestDevice> {
        let requirements = TestRequirements {
            extensions: vec![
                khr::video_queue::NAME,
                khr::video_encode_queue::NAME,
                khr::video_encode_h265::NAME,
            ],
            queue: Some(vk::QueueFlags::VIDEO_ENCODE_KHR),
        };

        TestDevice::with(&requirements).ok()
    }

    #[test]
    fn the_gpu_reports_what_the_encoder_relies_on() {
        let Some(device) = encode_device() else {
            return;
        };
        let capabilities = EncoderCapabilities::query(device.entry(), device.memory()).unwrap();

        assert!(capabilities.max_coded_extent.width >= 1920);
        assert!(!capabilities.ctb_sizes.is_empty());
        assert!(
            device.validation_messages().is_empty(),
            "{:?}",
            device.validation_messages()
        );
    }

    #[test]
    fn a_session_encodes_its_stream_headers() {
        let Some(device) = encode_device() else {
            return;
        };
        let capabilities = EncoderCapabilities::query(device.entry(), device.memory()).unwrap();
        let extent = vk::Extent2D {
            width: 1918,
            height: 1080,
        };
        let geometry = PictureGeometry::new(extent, &capabilities).unwrap();
        let family = device.extra_queue().unwrap().family;
        let session = EncodeSession::new(device.memory(), &capabilities, family, &geometry, 60).unwrap();

        assert!(session.headers.starts_with(&[0, 0, 0, 1]) || session.headers.starts_with(&[0, 0, 1]));
        assert!(
            device.validation_messages().is_empty(),
            "{:?}",
            device.validation_messages()
        );
    }

    #[test]
    fn frames_come_back_in_order_starting_with_a_keyframe() {
        let Some(device) = encode_device() else {
            return;
        };
        let memory = device.memory();
        let queue = device.extra_queue().unwrap();
        let encode_queue = EncodeQueue {
            family: queue.family,
            timeline: queue.timeline.clone(),
        };
        let extent = vk::Extent2D {
            width: 640,
            height: 360,
        };
        let graphics_family = 0;
        let mut encoder = VideoEncoder::new(
            device.entry(),
            memory,
            encode_queue,
            device.transfer(),
            graphics_family,
            extent,
            30,
        )
        .unwrap();
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.family);
        let pool = unsafe { memory.device().create_command_pool(&pool_info, None) }.unwrap();
        let pool = memory_vulkan::DeviceObject::new(memory, pool);
        let mut frames = Vec::new();

        for _ in 0..5 {
            let input = encoder.next_input().unwrap().expect("an input is free");
            let barrier = vk::ImageMemoryBarrier2::default()
                .dst_stage_mask(vk::PipelineStageFlags2::VIDEO_ENCODE_KHR)
                .dst_access_mask(vk::AccessFlags2::VIDEO_ENCODE_READ_KHR)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::VIDEO_ENCODE_SRC_KHR)
                .image(input.handle())
                .subresource_range(input.subresource_range());
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(pool.handle())
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = unsafe { memory.device().allocate_command_buffers(&allocate_info) }.unwrap()[0];
            let recording = queue.timeline.begin(command_buffer).unwrap();
            let barriers = [barrier];
            unsafe {
                memory.device().cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(&barriers),
                );
            }
            let ready = recording.submit(&[], &[]).unwrap();
            encoder.encode(&ready).unwrap();
            queue.timeline.wait(ready.value + 1).unwrap();
            device.frame().unwrap();
            frames.extend(encoder.poll().unwrap());
        }

        while !encoder.is_idle() {
            device.frame().unwrap();
            frames.extend(encoder.poll().unwrap());
        }

        assert_eq!(frames.len(), 5);
        assert!(frames[0].keyframe && !frames[1].keyframe);
        assert!(
            frames
                .iter()
                .all(|frame| frame.bytes.starts_with(&[0, 0, 1]) || frame.bytes.starts_with(&[0, 0, 0, 1]))
        );
        assert!(
            device.validation_messages().is_empty(),
            "{:#?}",
            device.validation_messages()
        );
    }

    #[test]
    fn rendered_frames_convert_and_encode() {
        let Some(device) = encode_device() else {
            return;
        };
        let memory = device.memory();
        let queue = device.extra_queue().unwrap();
        let extent = vk::Extent2D {
            width: 638,
            height: 360,
        };
        let graphics_family = 0;
        let encode_queue = EncodeQueue {
            family: queue.family,
            timeline: queue.timeline.clone(),
        };
        let mut encoder = VideoEncoder::new(
            device.entry(),
            memory,
            encode_queue,
            device.transfer(),
            graphics_family,
            extent,
            30,
        )
        .unwrap();
        let conversion = ColorConversion::new(memory, encoder.coded_extent(), 1).unwrap();
        let source = memory_vulkan::Image::new(
            memory,
            &memory_vulkan::ImageDescription {
                name: "rendered frame",
                extent,
                format: vk::Format::B8G8R8A8_SRGB,
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                kind: memory_vulkan::ImageKind::Single,
                memory: memory_vulkan::ImageMemory::Shared,
                sharing: memory_vulkan::QueueSharing::Exclusive,
            },
        )
        .unwrap();
        let rendered = ConversionSource {
            view: source.view(),
            format: vk::Format::B8G8R8A8_SRGB,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            extent,
        };
        let mut frames = Vec::new();

        for shade in [0.1, 0.5, 0.9] {
            let input = encoder.next_input().unwrap().expect("an input is free");
            let ready = device
                .run(|command_buffer| {
                    clear(
                        memory.device(),
                        command_buffer,
                        &source,
                        [shade, shade * 0.5, 1.0 - shade, 1.0],
                    );
                    conversion.record(command_buffer, 0, &rendered, input);
                })
                .unwrap();
            encoder.encode(&ready).unwrap();
        }

        while !encoder.is_idle() {
            device.frame().unwrap();
            frames.extend(encoder.poll().unwrap());
        }

        assert_eq!(frames.len(), 3);
        assert!(
            device.validation_messages().is_empty(),
            "{:#?}",
            device.validation_messages()
        );

        let path = std::env::temp_dir().join(format!("video-vulkan-test-{}.mp4", std::process::id()));
        let mut writer = Mp4Writer::create(&path, extent, 30, encoder.headers()).unwrap();

        for frame in &frames {
            writer.write(frame).unwrap();
        }

        writer.finish().unwrap();
        let probe = std::process::Command::new("ffprobe")
            .args(["-v", "error", "-count_frames", "-select_streams", "v:0"])
            .args([
                "-show_entries",
                "stream=codec_name,width,height,nb_read_frames",
                "-of",
                "csv=p=0",
            ])
            .arg(&path)
            .output();
        std::fs::remove_file(&path).unwrap();

        // An independent decoder reads the file back, where one is installed.
        if let Ok(probe) = probe {
            assert_eq!(
                String::from_utf8_lossy(&probe.stdout).trim(),
                "hevc,638,360,3",
                "{probe:?}"
            );
        }
    }

    #[test]
    fn frames_keep_their_own_bitstream_when_inputs_cycle_before_polling() {
        let Some(device) = encode_device() else {
            return;
        };
        let memory = device.memory();
        let queue = device.extra_queue().unwrap();
        let extent = vk::Extent2D {
            width: 320,
            height: 180,
        };
        let encode_queue = EncodeQueue {
            family: queue.family,
            timeline: queue.timeline.clone(),
        };
        let mut encoder =
            VideoEncoder::new(device.entry(), memory, encode_queue, device.transfer(), 0, extent, 30).unwrap();
        let conversion = ColorConversion::new(memory, encoder.coded_extent(), 1).unwrap();
        let source = memory_vulkan::Image::new(
            memory,
            &memory_vulkan::ImageDescription {
                name: "rendered frame",
                extent,
                format: vk::Format::B8G8R8A8_SRGB,
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                kind: memory_vulkan::ImageKind::Single,
                memory: memory_vulkan::ImageMemory::Shared,
                sharing: memory_vulkan::QueueSharing::Exclusive,
            },
        )
        .unwrap();
        let rendered = ConversionSource {
            view: source.view(),
            format: vk::Format::B8G8R8A8_SRGB,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            extent,
        };
        let shades: Vec<f32> = (0..12).map(|index| index as f32 / 11.0).collect();
        let mut frames = Vec::new();

        // Frames are only polled when no input is free, so inputs come back around before earlier frames are read.
        for shade in &shades {
            let input = loop {
                if let Some(input) = encoder.next_input().unwrap() {
                    break input;
                }

                device.frame().unwrap();
                frames.extend(encoder.poll().unwrap());
            };
            let ready = device
                .run(|command_buffer| {
                    clear(memory.device(), command_buffer, &source, [*shade, *shade, *shade, 1.0]);
                    conversion.record(command_buffer, 0, &rendered, input);
                })
                .unwrap();
            encoder.encode(&ready).unwrap();
        }

        while !encoder.is_idle() {
            device.frame().unwrap();
            frames.extend(encoder.poll().unwrap());
        }

        let path = std::env::temp_dir().join(format!("video-vulkan-cycle-{}.mp4", std::process::id()));
        let mut writer = Mp4Writer::create(&path, extent, 30, encoder.headers()).unwrap();

        for frame in &frames {
            writer.write(frame).unwrap();
        }

        writer.finish().unwrap();
        let decoded = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-i"])
            .arg(&path)
            .args(["-f", "rawvideo", "-pix_fmt", "gray", "-"])
            .output();
        std::fs::remove_file(&path).unwrap();

        // An independent decoder reads every frame back, where one is installed: each keeps its own shade.
        let Ok(decoded) = decoded else {
            return;
        };
        let frame_size = (extent.width * extent.height) as usize;
        assert_eq!(decoded.stdout.len(), frame_size * shades.len(), "{decoded:?}");
        let mut previous = 0;

        for (index, frame) in decoded.stdout.chunks(frame_size).enumerate() {
            let centre = frame[frame_size / 2 + extent.width as usize / 2];
            assert!(
                index == 0 || centre > previous,
                "frame {index} is not brighter than the one before it"
            );
            previous = centre;
        }
    }

    #[test]
    fn an_input_comes_back_only_once_its_frame_is_read() {
        let Some(device) = encode_device() else {
            return;
        };
        let memory = device.memory();
        let queue = device.extra_queue().unwrap();
        let extent = vk::Extent2D {
            width: 320,
            height: 180,
        };
        let encode_queue = EncodeQueue {
            family: queue.family,
            timeline: queue.timeline.clone(),
        };
        let mut encoder =
            VideoEncoder::new(device.entry(), memory, encode_queue, device.transfer(), 0, extent, 30).unwrap();
        let mut last = None;

        while let Some(input) = encoder.next_input().unwrap() {
            let ready = device
                .run(|command_buffer| prepare_input(memory.device(), command_buffer, input))
                .unwrap();
            encoder.encode(&ready).unwrap();
            last = Some(queue.timeline.clone());
        }

        let encode_timeline = last.expect("an input was free at first");
        let encoded = encode_timeline.completed().unwrap();
        encode_timeline.wait(encoded + 1).ok();
        device.frame().unwrap();

        // Every encode is done, but no frame has been read: the inputs still hold their frames' feedback.
        assert!(encoder.next_input().unwrap().is_none());

        let mut frames = Vec::new();

        while !encoder.is_idle() {
            device.frame().unwrap();
            frames.extend(encoder.poll().unwrap());
        }

        assert!(!frames.is_empty());
        assert!(encoder.next_input().unwrap().is_some());
    }

    fn prepare_input(device: &ash::Device, command_buffer: vk::CommandBuffer, input: &memory_vulkan::Image) {
        let barrier = [vk::ImageMemoryBarrier2::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::VIDEO_ENCODE_SRC_KHR)
            .image(input.handle())
            .subresource_range(input.subresource_range())];
        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&barrier),
            )
        };
    }

    fn clear(device: &ash::Device, command_buffer: vk::CommandBuffer, image: &memory_vulkan::Image, linear: [f32; 4]) {
        let range = image.subresource_range();
        let to_clear = [vk::ImageMemoryBarrier2::default()
            .dst_stage_mask(vk::PipelineStageFlags2::CLEAR)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(image.handle())
            .subresource_range(range)];
        let to_read = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::CLEAR)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
            .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(image.handle())
            .subresource_range(range)];
        let color = vk::ClearColorValue { float32: linear };

        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_clear),
            );
            device.cmd_clear_color_image(
                command_buffer,
                image.handle(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &color,
                &[range],
            );
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_read),
            );
        }
    }
}
