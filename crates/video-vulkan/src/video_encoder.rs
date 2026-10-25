use std::collections::VecDeque;
use std::ffi::CStr;

use ash::vk::native::{
    StdVideoH265PictureType_STD_VIDEO_H265_PICTURE_TYPE_IDR, StdVideoH265PictureType_STD_VIDEO_H265_PICTURE_TYPE_P,
};
use ash::{khr, vk};
use memory_vulkan::{
    Buffer, DeviceObject, Image, ImageDescription, ImageKind, ImageMemory, MemoryContext, MemoryLocation, QueueSharing,
    QueueTimeline, ReadbackTicket, TimelineValue, TransferPriority, TransferQueue, TransferStatus,
};

use crate::encode_feedback::EncodeFeedback;
use crate::encode_session::EncodeSession;
use crate::encoder_capabilities::EncoderCapabilities;
use crate::h265_profile::H265Profile;
use crate::picture_geometry::PictureGeometry;
use crate::picture_syntax::PictureSyntax;
use crate::reference_picture::ReferencePicture;
use crate::{EncodedFrame, VideoError};

// Frames being filled or encoded at once; a fourth waits for the oldest to finish.
const INPUT_COUNT: usize = 3;
const REFERENCE_COUNT: u32 = 2;
const KEYFRAME_INTERVAL_SECONDS: u32 = 2;
// Constant quantization: lower is finer. Rendered scenes keep stars and thin edges at this value.
const CONSTANT_QP: i32 = 20;

/// Encodes frames to H.265 on the GPU's encode queue. Each frame is written into the image `next_input` returns, as
/// NV12 in the `VIDEO_ENCODE_SRC_KHR` layout, by a submission `encode` then waits for; `poll` returns the finished
/// frames in order without waiting for the GPU.
pub struct VideoEncoder {
    memory: MemoryContext,
    queue: QueueTimeline,
    transfer: TransferQueue,
    bitstream_sharing: QueueSharing,
    geometry: PictureGeometry,
    keyframe_interval: u64,
    bitstream_size: u64,
    qp: i32,
    _command_pool: DeviceObject<vk::CommandPool>,
    inputs: Vec<InputSlot>,
    references: Vec<Image>,
    feedback: DeviceObject<vk::QueryPool>,
    session: EncodeSession,
    next_input: usize,
    frames: u64,
    reference: Option<ReferencePicture>,
    pending: VecDeque<PendingFrame>,
}

struct InputSlot {
    image: Image,
    command_buffer: vk::CommandBuffer,
}

struct PendingFrame {
    input: usize,
    done: TimelineValue,
    bitstream: ReadbackTicket,
    keyframe: bool,
    feedback: Option<EncodeFeedback>,
}

impl VideoEncoder {
    /// The device extensions an encoder needs.
    pub const DEVICE_EXTENSIONS: [&'static CStr; 3] = [
        khr::video_queue::NAME,
        khr::video_encode_queue::NAME,
        khr::video_encode_h265::NAME,
    ];
    /// The capability of the queue an encoder encodes on.
    pub const QUEUE_CAPABILITY: vk::QueueFlags = vk::QueueFlags::VIDEO_ENCODE_KHR;

    /// An encoder of `extent` frames, which must be even, at `fps` frames per second. Its inputs are shared with
    /// `graphics_family`, whose submissions fill them.
    pub fn new(
        entry: &ash::Entry,
        memory: &MemoryContext,
        queue: QueueTimeline,
        transfer: &TransferQueue,
        graphics_family: u32,
        extent: vk::Extent2D,
        fps: u32,
    ) -> Result<VideoEncoder, VideoError> {
        let capabilities = EncoderCapabilities::query(entry, memory)?;
        let geometry = PictureGeometry::new(extent, &capabilities)?;
        let session = EncodeSession::new(memory, &capabilities, queue.family, &geometry, fps)?;
        let device = memory.device();
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.family);
        let command_pool = DeviceObject::new(memory, unsafe { device.create_command_pool(&pool_info, None) }?);
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(INPUT_COUNT as u32);
        let command_buffers = unsafe { device.allocate_command_buffers(&allocate_info) }?;
        let input_sharing = QueueSharing::Concurrent(vec![graphics_family, queue.family]);
        let format = capabilities.picture_format();
        let inputs = command_buffers
            .into_iter()
            .map(|command_buffer| {
                let image = VideoEncoder::picture(
                    memory,
                    "video input",
                    &geometry,
                    format,
                    vk::ImageUsageFlags::VIDEO_ENCODE_SRC_KHR | vk::ImageUsageFlags::TRANSFER_DST,
                    input_sharing.clone(),
                )?;

                Ok(InputSlot { image, command_buffer })
            })
            .collect::<Result<Vec<_>, VideoError>>()?;
        let references = (0..REFERENCE_COUNT)
            .map(|_| {
                VideoEncoder::picture(
                    memory,
                    "video reference",
                    &geometry,
                    format,
                    vk::ImageUsageFlags::VIDEO_ENCODE_DPB_KHR,
                    QueueSharing::Exclusive,
                )
            })
            .collect::<Result<Vec<_>, VideoError>>()?;
        let feedback = VideoEncoder::feedback_pool(memory)?;
        let mut families = match transfer.sharing() {
            QueueSharing::Concurrent(families) => families,
            QueueSharing::Exclusive => Vec::new(),
        };
        families.extend([graphics_family, queue.family]);
        families.sort_unstable();
        families.dedup();
        let raw_frame_size = u64::from(geometry.coded_extent.width) * u64::from(geometry.coded_extent.height) * 3 / 2;

        Ok(VideoEncoder {
            memory: memory.clone(),
            queue,
            transfer: transfer.clone(),
            bitstream_sharing: QueueSharing::Concurrent(families),
            geometry,
            keyframe_interval: u64::from(fps * KEYFRAME_INTERVAL_SECONDS),
            bitstream_size: raw_frame_size.next_multiple_of(capabilities.bitstream_size_alignment),
            qp: CONSTANT_QP.clamp(capabilities.min_qp, capabilities.max_qp),
            _command_pool: command_pool,
            inputs,
            references,
            feedback,
            session,
            next_input: 0,
            frames: 0,
            reference: None,
            pending: VecDeque::new(),
        })
    }

    /// The stream's VPS, SPS and PPS, in Annex B form.
    pub fn headers(&self) -> &[u8] {
        &self.session.headers
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.geometry.extent
    }

    /// The size the input is coded at: the extent rounded up to the encoder's granularity. Pixels past the extent are
    /// cropped away and only need to be written so they compress well, by repeating the edge.
    pub fn coded_extent(&self) -> vk::Extent2D {
        self.geometry.coded_extent
    }

    /// The image the next frame is written into, or `None` while every input still holds a frame not yet read
    /// back: its feedback query is reused with it.
    pub fn next_input(&self) -> Result<Option<&Image>, VideoError> {
        let unread = self
            .pending
            .iter()
            .any(|frame| frame.input == self.next_input && frame.feedback.is_none());

        Ok((!unread).then_some(&self.inputs[self.next_input].image))
    }

    /// Encodes the image `next_input` returned once `ready`, the submission that wrote it, completes.
    pub fn encode(&mut self, ready: &TimelineValue) -> Result<(), VideoError> {
        let input = self.next_input;
        let keyframe = self.frames.is_multiple_of(self.keyframe_interval);
        let reference = if keyframe { None } else { self.reference };
        let written = ReferencePicture {
            slot: reference.map_or(0, |reference| (reference.slot + 1) % REFERENCE_COUNT),
            order: reference.map_or(0, |reference| reference.order + 1),
            picture_type: if keyframe {
                StdVideoH265PictureType_STD_VIDEO_H265_PICTURE_TYPE_IDR
            } else {
                StdVideoH265PictureType_STD_VIDEO_H265_PICTURE_TYPE_P
            },
        };
        let bitstream = H265Profile::with_list(|list| {
            Buffer::new_extended(
                &self.memory,
                "video bitstream",
                self.bitstream_size,
                vk::BufferUsageFlags::VIDEO_ENCODE_DST_KHR | vk::BufferUsageFlags::TRANSFER_SRC,
                MemoryLocation::GpuOnly,
                &self.bitstream_sharing,
                list,
            )
        })?;
        let command_buffer = self.inputs[input].command_buffer;
        let recording = self.queue.timeline.begin(command_buffer)?;
        self.record(command_buffer, input, &bitstream, &written, reference.as_ref());
        let done = recording.submit(&[ready.wait_info(vk::PipelineStageFlags2::VIDEO_ENCODE_KHR)], &[])?;
        let bitstream = self
            .transfer
            .read_buffer(bitstream, Some(done.clone()), TransferPriority::High);
        self.pending.push_back(PendingFrame {
            input,
            done,
            bitstream,
            keyframe,
            feedback: None,
        });
        self.reference = Some(written);
        self.next_input = (input + 1) % INPUT_COUNT;
        self.frames += 1;

        Ok(())
    }

    /// The frames encoded and read back so far, in order. Reading each finished frame's feedback frees its input.
    pub fn poll(&mut self) -> Result<Vec<EncodedFrame>, VideoError> {
        for frame in &mut self.pending {
            if frame.feedback.is_some() {
                continue;
            }

            // Submissions complete in order: nothing after an unfinished one is finished either.
            if !frame.done.is_complete()? {
                break;
            }

            match VideoEncoder::read_feedback(&self.memory, &self.feedback, frame.input)? {
                Some(feedback) => frame.feedback = Some(feedback),
                None => break,
            }
        }

        let mut finished = Vec::new();

        while let Some(frame) = self.pending.front_mut() {
            let Some(feedback) = frame.feedback else {
                break;
            };
            let bytes = match frame.bitstream.poll() {
                TransferStatus::Pending => break,
                TransferStatus::Complete(bytes) => bytes,
                TransferStatus::Failed(error) => return Err(error.into()),
            };
            let start = feedback.offset as usize;
            let end = start + feedback.bytes as usize;
            let keyframe = frame.keyframe;
            self.pending.pop_front();
            finished.push(EncodedFrame {
                bytes: bytes[start..end].to_vec(),
                keyframe,
            });
        }

        Ok(finished)
    }

    /// Whether every frame given to `encode` has come back from `poll`.
    pub fn is_idle(&self) -> bool {
        self.pending.is_empty()
    }

    fn record(
        &self,
        command_buffer: vk::CommandBuffer,
        input: usize,
        bitstream: &Buffer,
        written: &ReferencePicture,
        reference: Option<&ReferencePicture>,
    ) {
        let device = self.memory.device();
        let first = self.frames == 0;

        if first {
            self.prepare_references(command_buffer);
        }

        unsafe { device.cmd_reset_query_pool(command_buffer, self.feedback.handle(), input as u32, 1) };
        self.begin_coding(command_buffer, written, reference, first);

        if first {
            self.reset_coding(command_buffer);
        }

        self.encode_picture(command_buffer, input, bitstream, written, reference);
        let end = vk::VideoEndCodingInfoKHR::default();
        unsafe { (self.session.functions.fp().cmd_end_video_coding_khr)(command_buffer, &end) };
    }

    // Binds the slot the frame is written to and the one it predicts from.
    fn begin_coding(
        &self,
        command_buffer: vk::CommandBuffer,
        written: &ReferencePicture,
        reference: Option<&ReferencePicture>,
        first: bool,
    ) {
        let written_resource = self.reference_resource(written.slot);
        let reference_resource = reference.map(|reference| self.reference_resource(reference.slot));
        // The slot being written is bound without an index: it holds nothing yet, or a picture no longer used.
        let mut bound = vec![
            vk::VideoReferenceSlotInfoKHR::default()
                .slot_index(-1)
                .picture_resource(&written_resource),
        ];

        if let (Some(reference), Some(resource)) = (reference, &reference_resource) {
            bound.push(
                vk::VideoReferenceSlotInfoKHR::default()
                    .slot_index(reference.slot as i32)
                    .picture_resource(resource),
            );
        }

        let mut rate_control = vk::VideoEncodeRateControlInfoKHR::default()
            .rate_control_mode(vk::VideoEncodeRateControlModeFlagsKHR::DISABLED);
        let mut begin = vk::VideoBeginCodingInfoKHR::default()
            .video_session(self.session.session.get().handle)
            .video_session_parameters(self.session.parameters.get().handle)
            .reference_slots(&bound);

        // Rate control is in its default state until the first frame's reset configures it.
        if !first {
            begin = begin.push_next(&mut rate_control);
        }

        unsafe { (self.session.functions.fp().cmd_begin_video_coding_khr)(command_buffer, &begin) };
    }

    // Starts the session's state: constant quality at the session's quality level.
    fn reset_coding(&self, command_buffer: vk::CommandBuffer) {
        let mut rate_control = vk::VideoEncodeRateControlInfoKHR::default()
            .rate_control_mode(vk::VideoEncodeRateControlModeFlagsKHR::DISABLED);
        let mut quality = vk::VideoEncodeQualityLevelInfoKHR::default().quality_level(self.session.quality_level);
        let control = vk::VideoCodingControlInfoKHR::default()
            .flags(
                vk::VideoCodingControlFlagsKHR::RESET
                    | vk::VideoCodingControlFlagsKHR::ENCODE_RATE_CONTROL
                    | vk::VideoCodingControlFlagsKHR::ENCODE_QUALITY_LEVEL,
            )
            .push_next(&mut rate_control)
            .push_next(&mut quality);
        unsafe { (self.session.functions.fp().cmd_control_video_coding_khr)(command_buffer, &control) };
    }

    // Encodes the input into `bitstream`, reporting where it went in the input's feedback query.
    fn encode_picture(
        &self,
        command_buffer: vk::CommandBuffer,
        input: usize,
        bitstream: &Buffer,
        written: &ReferencePicture,
        reference: Option<&ReferencePicture>,
    ) {
        let device = self.memory.device();
        let syntax = PictureSyntax::new(written, reference);
        let picture = syntax.picture();
        let written_resource = self.reference_resource(written.slot);
        let reference_resource = reference.map(|reference| self.reference_resource(reference.slot));
        let input_resource = self.picture_resource(self.inputs[input].image.view());
        let mut written_slot_info =
            vk::VideoEncodeH265DpbSlotInfoKHR::default().std_reference_info(&syntax.written_reference);
        let mut reference_slot_info =
            vk::VideoEncodeH265DpbSlotInfoKHR::default().std_reference_info(&syntax.reference);
        let mut references = Vec::new();

        if let (Some(reference), Some(resource)) = (reference, &reference_resource) {
            references.push(
                vk::VideoReferenceSlotInfoKHR::default()
                    .slot_index(reference.slot as i32)
                    .picture_resource(resource)
                    .push_next(&mut reference_slot_info),
            );
        }

        let slices = [vk::VideoEncodeH265NaluSliceSegmentInfoKHR::default()
            .constant_qp(self.qp)
            .std_slice_segment_header(&syntax.slice_header)];
        let mut h265 = vk::VideoEncodeH265PictureInfoKHR::default()
            .nalu_slice_segment_entries(&slices)
            .std_picture_info(&picture);
        let written_slot = vk::VideoReferenceSlotInfoKHR::default()
            .slot_index(written.slot as i32)
            .picture_resource(&written_resource)
            .push_next(&mut written_slot_info);
        let encode = vk::VideoEncodeInfoKHR::default()
            .dst_buffer(bitstream.handle())
            .dst_buffer_range(bitstream.size())
            .src_picture_resource(input_resource)
            .setup_reference_slot(&written_slot)
            .reference_slots(&references)
            .push_next(&mut h265);

        unsafe {
            device.cmd_begin_query(
                command_buffer,
                self.feedback.handle(),
                input as u32,
                vk::QueryControlFlags::empty(),
            );
            (self.session.encode_functions.fp().cmd_encode_video_khr)(command_buffer, &encode);
            device.cmd_end_query(command_buffer, self.feedback.handle(), input as u32);
        }
    }

    fn reference_resource(&self, slot: u32) -> vk::VideoPictureResourceInfoKHR<'static> {
        self.picture_resource(self.references[slot as usize].view())
    }

    fn picture_resource(&self, view: vk::ImageView) -> vk::VideoPictureResourceInfoKHR<'static> {
        vk::VideoPictureResourceInfoKHR::default()
            .coded_extent(self.geometry.coded_extent)
            .image_view_binding(view)
    }

    // The reference pictures start undefined; the encoder keeps them in the reference layout from then on.
    fn prepare_references(&self, command_buffer: vk::CommandBuffer) {
        let barriers: Vec<vk::ImageMemoryBarrier2> = self
            .references
            .iter()
            .map(|reference| {
                vk::ImageMemoryBarrier2::default()
                    .dst_stage_mask(vk::PipelineStageFlags2::VIDEO_ENCODE_KHR)
                    .dst_access_mask(vk::AccessFlags2::VIDEO_ENCODE_READ_KHR | vk::AccessFlags2::VIDEO_ENCODE_WRITE_KHR)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::VIDEO_ENCODE_DPB_KHR)
                    .image(reference.handle())
                    .subresource_range(reference.subresource_range())
            })
            .collect();
        let dependency = vk::DependencyInfo::default().image_memory_barriers(&barriers);
        unsafe { self.memory.device().cmd_pipeline_barrier2(command_buffer, &dependency) };
    }

    fn picture(
        memory: &MemoryContext,
        name: &str,
        geometry: &PictureGeometry,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        sharing: QueueSharing,
    ) -> Result<Image, VideoError> {
        let description = ImageDescription {
            name,
            extent: geometry.coded_extent,
            format,
            usage,
            aspect: vk::ImageAspectFlags::COLOR,
            mip_levels: 1,
            kind: ImageKind::Single,
            memory: ImageMemory::Dedicated,
            sharing,
        };

        Ok(H265Profile::with_list(|list| {
            Image::new_extended(memory, &description, list)
        })?)
    }

    fn feedback_pool(memory: &MemoryContext) -> Result<DeviceObject<vk::QueryPool>, VideoError> {
        let pool = H265Profile::with(|profile| {
            let mut profile = *profile;
            let mut feedback = vk::QueryPoolVideoEncodeFeedbackCreateInfoKHR::default().encode_feedback_flags(
                vk::VideoEncodeFeedbackFlagsKHR::BITSTREAM_BUFFER_OFFSET
                    | vk::VideoEncodeFeedbackFlagsKHR::BITSTREAM_BYTES_WRITTEN,
            );
            let info = vk::QueryPoolCreateInfo::default()
                .query_type(vk::QueryType::VIDEO_ENCODE_FEEDBACK_KHR)
                .query_count(INPUT_COUNT as u32)
                .push_next(&mut feedback)
                .push_next(&mut profile);

            unsafe { memory.device().create_query_pool(&info, None) }
        })?;

        Ok(DeviceObject::new(memory, pool))
    }

    // `None` while the result is not available yet, which can outlast the submission's completion.
    fn read_feedback(
        memory: &MemoryContext,
        pool: &DeviceObject<vk::QueryPool>,
        input: usize,
    ) -> Result<Option<EncodeFeedback>, VideoError> {
        let mut results = [EncodeFeedback::default()];
        let read = unsafe {
            memory.device().get_query_pool_results(
                pool.handle(),
                input as u32,
                &mut results,
                vk::QueryResultFlags::WITH_STATUS_KHR,
            )
        };

        match read {
            Ok(()) => {}
            Err(vk::Result::NOT_READY) => return Ok(None),
            Err(error) => return Err(error.into()),
        }

        let feedback = results[0];

        if feedback.status != vk::QueryResultStatusKHR::COMPLETE.as_raw() {
            return Err(VideoError::Encoding(feedback.status));
        }

        Ok(Some(feedback))
    }
}
