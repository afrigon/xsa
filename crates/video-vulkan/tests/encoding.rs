use std::path::Path;
use std::process::Command;

use ash::vk;
use memory_vulkan::{
    Image, ImageDescription, ImageKind, ImageMemory, QueueSharing, QueueTimeline, TestDevice, TestRequirements,
    TimelineValue,
};
use video_vulkan::{ColorConversion, ConversionSource, EncodedFrame, Mp4Writer, VideoEncoder};

const EXTENT: vk::Extent2D = vk::Extent2D {
    width: 318,
    height: 180,
};
const FPS: u32 = 30;
const RENDERED_FORMAT: vk::Format = vk::Format::B8G8R8A8_SRGB;

// An encoder fed by a rendered frame the tests clear to a grey, on a device that can encode.
struct Fixture {
    encoder: VideoEncoder,
    conversion: ColorConversion,
    source: Image,
    frames: Vec<EncodedFrame>,
    device: TestDevice,
}

impl Fixture {
    // `None` on a GPU without an H.265 encoder.
    fn new() -> Option<Fixture> {
        let requirements = TestRequirements {
            extensions: VideoEncoder::DEVICE_EXTENSIONS.to_vec(),
            queue: Some(VideoEncoder::QUEUE_CAPABILITY),
        };
        let device = TestDevice::with(&requirements).ok()?;
        let memory = device.memory();
        let queue = device
            .extra_queue()
            .expect("the device has the encode queue it was asked for");
        let queue = QueueTimeline {
            family: queue.family,
            timeline: queue.timeline.clone(),
        };
        let graphics_family = 0;
        let encoder = VideoEncoder::new(
            device.entry(),
            memory,
            queue,
            device.transfer(),
            graphics_family,
            EXTENT,
            FPS,
        )
        .unwrap();
        let conversion = ColorConversion::new(memory, encoder.coded_extent(), 1).unwrap();
        let source = Image::new(
            memory,
            &ImageDescription {
                name: "rendered frame",
                extent: EXTENT,
                format: RENDERED_FORMAT,
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                kind: ImageKind::Single,
                memory: ImageMemory::Shared,
                sharing: QueueSharing::Exclusive,
            },
        )
        .unwrap();

        Some(Fixture {
            encoder,
            conversion,
            source,
            frames: Vec::new(),
            device,
        })
    }

    // Renders a frame of one grey and encodes it, polling only while no input is free.
    fn encode_grey(&mut self, grey: f32) {
        while self.encoder.next_input().unwrap().is_none() {
            self.poll();
        }

        let ready = self.render(grey);
        self.encoder.encode(&ready).unwrap();
    }

    fn render(&self, grey: f32) -> TimelineValue {
        let input = self.encoder.next_input().unwrap().expect("an input is free");
        let source = ConversionSource {
            view: self.source.view(),
            format: RENDERED_FORMAT,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            extent: EXTENT,
        };

        self.device
            .run(|command_buffer| {
                Fixture::clear(self.device.memory().device(), command_buffer, &self.source, grey);
                self.conversion.record(command_buffer, 0, &source, input);
            })
            .unwrap()
    }

    fn poll(&mut self) {
        self.device.frame().unwrap();
        self.frames.extend(self.encoder.poll().unwrap());
    }

    fn finish(&mut self) {
        while !self.encoder.is_idle() {
            self.poll();
        }
    }

    fn write(&self, path: &Path) {
        let mut writer = Mp4Writer::create(path, EXTENT, FPS, self.encoder.headers()).unwrap();

        for frame in &self.frames {
            writer.write(frame).unwrap();
        }

        writer.finish().unwrap();
    }

    fn assert_no_validation_messages(&self) {
        let messages = self.device.validation_messages();
        assert!(messages.is_empty(), "{messages:#?}");
    }

    fn clear(device: &ash::Device, command_buffer: vk::CommandBuffer, image: &Image, grey: f32) {
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
        let color = vk::ClearColorValue {
            float32: [grey, grey, grey, 1.0],
        };

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

#[test]
fn every_frame_keeps_its_own_picture_in_the_file() {
    let Some(mut fixture) = Fixture::new() else {
        return;
    };
    let greys: Vec<f32> = (0..12).map(|index| index as f32 / 11.0).collect();

    // More frames than inputs, so inputs come back around while earlier frames are still being read back.
    for grey in &greys {
        fixture.encode_grey(*grey);
    }

    fixture.finish();
    fixture.assert_no_validation_messages();
    assert_eq!(fixture.frames.len(), greys.len());
    assert!(fixture.frames[0].keyframe && !fixture.frames[1].keyframe);

    let path = std::env::temp_dir().join(format!("video-vulkan-test-{}.mp4", std::process::id()));
    fixture.write(&path);
    let probe = Command::new("ffprobe")
        .args(["-v", "error", "-count_frames", "-select_streams", "v:0"])
        .args([
            "-show_entries",
            "stream=codec_name,width,height,nb_read_frames",
            "-of",
            "csv=p=0",
        ])
        .arg(&path)
        .output();
    let decoded = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&path)
        .args(["-f", "rawvideo", "-pix_fmt", "gray", "-"])
        .output();
    std::fs::remove_file(&path).unwrap();

    // An independent decoder reads the file back, where one is installed.
    let (Ok(probe), Ok(decoded)) = (probe, decoded) else {
        return;
    };
    let expected = format!("hevc,{},{},{}", EXTENT.width, EXTENT.height, greys.len());
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), expected, "{probe:?}");
    let frame_size = (EXTENT.width * EXTENT.height) as usize;
    let centre = frame_size / 2 + EXTENT.width as usize / 2;
    let brightness: Vec<u8> = decoded.stdout.chunks(frame_size).map(|frame| frame[centre]).collect();
    assert!(
        brightness.windows(2).all(|pair| pair[0] < pair[1]),
        "each frame is brighter than the one before: {brightness:?}"
    );
}

#[test]
fn an_input_comes_back_only_once_its_frame_is_read() {
    let Some(mut fixture) = Fixture::new() else {
        return;
    };
    let mut encodes = 0;

    while fixture.encoder.next_input().unwrap().is_some() {
        let ready = fixture.render(0.5);
        fixture.encoder.encode(&ready).unwrap();
        encodes += 1;
    }

    // The encode timeline signals one value per encode: once all are done, no frame has been read yet, and each
    // input still holds its frame's feedback.
    fixture.device.extra_queue().unwrap().timeline.wait(encodes).unwrap();
    fixture.device.frame().unwrap();
    assert!(fixture.encoder.next_input().unwrap().is_none());

    fixture.finish();
    assert!(fixture.encoder.next_input().unwrap().is_some());
    fixture.assert_no_validation_messages();
}
