use std::fs;
use std::path::{Path, PathBuf};

use ash::vk;
use memory_vulkan::TimelineValue;
use video_vulkan::{ColorConversion, ConversionSource, EncodeQueue, Mp4Writer, VideoEncoder, VideoError};

use super::gpu_context::GpuContext;
use super::video_phase::VideoPhase;

// A video being written: frames convert and encode as they render, and their bitstream streams into the file.
pub(super) struct VideoOutput {
    pub phase: VideoPhase,
    path: PathBuf,
    source_format: vk::Format,
    conversion: ColorConversion,
    encoder: VideoEncoder,
    writer: Mp4Writer,
    failure: Option<VideoError>,
}

impl VideoOutput {
    pub fn new(
        gpu: &GpuContext,
        path: PathBuf,
        source_extent: vk::Extent2D,
        source_format: vk::Format,
        fps: u32,
        frames_in_flight: usize,
    ) -> Result<VideoOutput, VideoError> {
        // 4:2:0 frames have an even size: an odd window leaves out its last column or row.
        let extent = vk::Extent2D {
            width: source_extent.width & !1,
            height: source_extent.height & !1,
        };
        let queue = EncodeQueue {
            family: gpu.device.encode_family(),
            timeline: gpu.encode.clone(),
        };
        let encoder = VideoEncoder::new(
            gpu.entry(),
            &gpu.memory,
            queue,
            &gpu.transfer,
            gpu.device.queue_family(),
            extent,
            fps,
        )?;
        let conversion = ColorConversion::new(&gpu.memory, encoder.coded_extent(), frames_in_flight)?;
        let writer = Mp4Writer::create(&path, extent, fps, encoder.headers())?;

        Ok(VideoOutput {
            phase: VideoPhase::Recording,
            path,
            source_format,
            conversion,
            encoder,
            writer,
            failure: None,
        })
    }

    // Whether the next frame has an input to convert into; a failure lets frames through so it can be reported.
    pub fn is_ready(&self) -> bool {
        self.phase == VideoPhase::Closing || !matches!(self.encoder.next_input(), Ok(None))
    }

    // Converts the frame sampled through `view` in `layout`, of `extent`, into the encoder's next input.
    pub fn record_conversion(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_slot: usize,
        view: vk::ImageView,
        layout: vk::ImageLayout,
        extent: vk::Extent2D,
    ) -> Result<(), VideoError> {
        let input = self
            .encoder
            .next_input()?
            .ok_or_else(|| VideoError::Unsupported("a frame while every input is encoding".to_string()))?;
        let source = ConversionSource {
            view,
            format: self.source_format,
            layout,
            extent,
        };
        self.conversion.record(command_buffer, frame_slot, &source, input);

        Ok(())
    }

    pub fn encode(&mut self, rendered: &TimelineValue) {
        if let Err(error) = self.encoder.encode(rendered) {
            self.failure.get_or_insert(error);
        }
    }

    // Writes the frames encoded so far, and reports a failure once.
    pub fn write_encoded(&mut self) -> Result<(), VideoError> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }

        for frame in self.encoder.poll()? {
            self.writer.write(&frame)?;
        }

        Ok(())
    }

    pub fn is_done(&self) -> bool {
        self.phase == VideoPhase::Closing && self.encoder.is_idle()
    }

    // A file that fails to close is incomplete, so it is deleted.
    pub fn finish(self) -> Result<PathBuf, VideoError> {
        let path = self.path;

        match self.writer.finish() {
            Ok(()) => Ok(path),
            Err(error) => {
                VideoOutput::delete(&path);

                Err(error)
            }
        }
    }

    // Stops at once and deletes the file, which holds a capture that failed.
    pub fn discard(self) {
        let path = self.path;
        drop(self.writer);
        VideoOutput::delete(&path);
    }

    fn delete(path: &Path) {
        if let Err(error) = fs::remove_file(path) {
            tracing::warn!("deleting the failed capture {}: {error}", path.display());
        }
    }
}
