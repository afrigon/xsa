use std::collections::VecDeque;
use std::mem;

use ash::vk;
use ash::vk::native::{
    StdVideoEncodeH265PictureInfo, StdVideoEncodeH265ReferenceInfo, StdVideoEncodeH265ReferenceListsInfo,
    StdVideoEncodeH265SliceSegmentHeader, StdVideoH265PictureType,
    StdVideoH265PictureType_STD_VIDEO_H265_PICTURE_TYPE_IDR, StdVideoH265PictureType_STD_VIDEO_H265_PICTURE_TYPE_P,
    StdVideoH265ShortTermRefPicSet, StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_I,
    StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_P,
};
use memory_vulkan::{
    Buffer, DeviceObject, Image, ImageDescription, ImageKind, ImageMemory, MemoryContext, MemoryLocation, QueueSharing,
    ReadbackTicket, TimelineValue, TransferPriority, TransferQueue, TransferStatus,
};

use crate::encode_feedback::EncodeFeedback;
use crate::encode_session::EncodeSession;
use crate::encoder_capabilities::EncoderCapabilities;
use crate::h265_profile::H265Profile;
use crate::picture_geometry::PictureGeometry;
use crate::{EncodeQueue, EncodedFrame, VideoError};

// Frames being filled or encoded at once; a fourth waits for the oldest to finish.
const INPUT_COUNT: usize = 3;
const REFERENCE_COUNT: u32 = 2;
const KEYFRAME_INTERVAL_SECONDS: u32 = 2;
// Constant quantization: lower is finer. Rendered scenes keep stars and thin edges at this value.
const CONSTANT_QP: i32 = 20;
// H.265: the slice header may name up to five merge candidates, sent as five minus the count.
const MAX_MERGE_CANDIDATES: u8 = 5;
// Std reference lists mark unused entries with this index.
const NO_REFERENCE_PICTURE: u8 = 0xFF;

/// Encodes frames to H.265 on the GPU's encode queue. Each frame is written into the image `next_input` returns, as
/// NV12 in the `VIDEO_ENCODE_SRC_KHR` layout, by a submission `encode` then waits for; `poll` returns the finished
/// frames in order without waiting for the GPU.
pub struct VideoEncoder {
    memory: MemoryContext,
    queue: EncodeQueue,
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
    done: Option<TimelineValue>,
}

// The picture the next frame predicts from: its slot in the decoded picture buffer and its order in the stream.
#[derive(Clone, Copy)]
struct ReferencePicture {
    slot: u32,
    order: i32,
    picture_type: StdVideoH265PictureType,
}

struct PendingFrame {
    input: usize,
    done: TimelineValue,
    bitstream: ReadbackTicket,
    keyframe: bool,
    feedback: Option<EncodeFeedback>,
}

impl VideoEncoder {
    /// An encoder of `extent` frames, which must be even, at `fps` frames per second. Its inputs are shared with
    /// `graphics_family`, whose submissions fill them.
    pub fn new(
        entry: &ash::Entry,
        memory: &MemoryContext,
        queue: EncodeQueue,
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

                Ok(InputSlot {
                    image,
                    command_buffer,
                    done: None,
                })
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

    /// The image the next frame is written into, or `None` while every input is still being encoded.
    pub fn next_input(&self) -> Result<Option<&Image>, VideoError> {
        let slot = &self.inputs[self.next_input];
        let free = match &slot.done {
            Some(done) => done.is_complete()?,
            None => true,
        };

        Ok(free.then_some(&slot.image))
    }

    /// Encodes the image `next_input` returned once `ready`, the submission that wrote it, completes.
    pub fn encode(&mut self, ready: &TimelineValue) -> Result<(), VideoError> {
        let input = self.next_input;
        let keyframe = self.frames.is_multiple_of(self.keyframe_interval);
        let reference = if keyframe { None } else { self.reference };
        let setup = ReferencePicture {
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
        self.record(command_buffer, input, &bitstream, &setup, reference.as_ref());
        let done = recording.submit(&[ready.wait_info(vk::PipelineStageFlags2::VIDEO_ENCODE_KHR)], &[])?;
        let bitstream = self
            .transfer
            .read_buffer(bitstream, Some(done.clone()), TransferPriority::High);
        self.inputs[input].done = Some(done.clone());
        self.pending.push_back(PendingFrame {
            input,
            done,
            bitstream,
            keyframe,
            feedback: None,
        });
        self.reference = Some(setup);
        self.next_input = (input + 1) % INPUT_COUNT;
        self.frames += 1;

        Ok(())
    }

    /// The frames encoded and read back so far, in order.
    pub fn poll(&mut self) -> Result<Vec<EncodedFrame>, VideoError> {
        let mut finished = Vec::new();

        while let Some(frame) = self.pending.front_mut() {
            if frame.feedback.is_none() {
                if !frame.done.is_complete()? {
                    break;
                }

                frame.feedback = Some(VideoEncoder::read_feedback(&self.memory, &self.feedback, frame.input)?);
            }

            let bytes = match frame.bitstream.poll() {
                TransferStatus::Pending => break,
                TransferStatus::Complete(bytes) => bytes,
                TransferStatus::Failed(error) => return Err(error.into()),
            };
            let feedback = frame.feedback.expect("the feedback was read above");
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
        setup: &ReferencePicture,
        reference: Option<&ReferencePicture>,
    ) {
        let device = self.memory.device();
        let functions = self.session.functions.fp();
        let first = self.frames == 0;

        if first {
            self.prepare_references(command_buffer);
        }

        unsafe { device.cmd_reset_query_pool(command_buffer, self.feedback.handle(), input as u32, 1) };
        let coded_extent = self.geometry.coded_extent;
        let resource = |view: vk::ImageView| {
            vk::VideoPictureResourceInfoKHR::default()
                .coded_extent(coded_extent)
                .image_view_binding(view)
        };
        let setup_resource = resource(self.references[setup.slot as usize].view());
        let reference_resource = reference.map(|reference| resource(self.references[reference.slot as usize].view()));
        let input_resource = resource(self.inputs[input].image.view());
        // Zeroed Std structures leave every flag and pointer unset; the fields that matter are set below.
        let mut setup_info: StdVideoEncodeH265ReferenceInfo = unsafe { mem::zeroed() };
        setup_info.pic_type = setup.picture_type;
        setup_info.PicOrderCntVal = setup.order;
        let mut setup_slot_info = vk::VideoEncodeH265DpbSlotInfoKHR::default().std_reference_info(&setup_info);
        let mut reference_info: StdVideoEncodeH265ReferenceInfo = unsafe { mem::zeroed() };

        if let Some(reference) = reference {
            reference_info.pic_type = reference.picture_type;
            reference_info.PicOrderCntVal = reference.order;
        }

        let mut reference_slot_info = vk::VideoEncodeH265DpbSlotInfoKHR::default().std_reference_info(&reference_info);
        // The slot being written is bound without an index: it holds nothing yet, or a picture no longer used.
        let mut bound = vec![
            vk::VideoReferenceSlotInfoKHR::default()
                .slot_index(-1)
                .picture_resource(&setup_resource),
        ];
        let mut references = Vec::new();

        if let (Some(reference), Some(resource)) = (reference, &reference_resource) {
            bound.push(
                vk::VideoReferenceSlotInfoKHR::default()
                    .slot_index(reference.slot as i32)
                    .picture_resource(resource),
            );
            references.push(
                vk::VideoReferenceSlotInfoKHR::default()
                    .slot_index(reference.slot as i32)
                    .picture_resource(resource)
                    .push_next(&mut reference_slot_info),
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

        unsafe { (functions.cmd_begin_video_coding_khr)(command_buffer, &begin) };

        if first {
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
            unsafe { (functions.cmd_control_video_coding_khr)(command_buffer, &control) };
        }

        let mut slice_header: StdVideoEncodeH265SliceSegmentHeader = unsafe { mem::zeroed() };
        slice_header.MaxNumMergeCand = MAX_MERGE_CANDIDATES;
        let mut lists: StdVideoEncodeH265ReferenceListsInfo = unsafe { mem::zeroed() };
        lists.RefPicList0 = [NO_REFERENCE_PICTURE; 15];
        lists.RefPicList1 = [NO_REFERENCE_PICTURE; 15];
        let mut reference_set: StdVideoH265ShortTermRefPicSet = unsafe { mem::zeroed() };

        match reference {
            Some(reference) => {
                slice_header.slice_type = StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_P;
                lists.RefPicList0[0] = reference.slot as u8;
                // The previous picture, one step back in output order.
                reference_set.num_negative_pics = 1;
                reference_set.used_by_curr_pic_s0_flag = 1;
            }
            None => slice_header.slice_type = StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_I,
        }

        let mut picture: StdVideoEncodeH265PictureInfo = unsafe { mem::zeroed() };
        picture.pic_type = setup.picture_type;
        picture.PicOrderCntVal = setup.order;
        picture.flags.set_is_reference(1);
        picture.flags.set_pic_output_flag(1);
        picture.pShortTermRefPicSet = &reference_set;

        if reference.is_some() {
            picture.pRefLists = &lists;
        } else {
            picture.flags.set_IrapPicFlag(1);
        }

        let slices = [vk::VideoEncodeH265NaluSliceSegmentInfoKHR::default()
            .constant_qp(self.qp)
            .std_slice_segment_header(&slice_header)];
        let mut h265 = vk::VideoEncodeH265PictureInfoKHR::default()
            .nalu_slice_segment_entries(&slices)
            .std_picture_info(&picture);
        let setup_slot = vk::VideoReferenceSlotInfoKHR::default()
            .slot_index(setup.slot as i32)
            .picture_resource(&setup_resource)
            .push_next(&mut setup_slot_info);
        let encode = vk::VideoEncodeInfoKHR::default()
            .dst_buffer(bitstream.handle())
            .dst_buffer_range(bitstream.size())
            .src_picture_resource(input_resource)
            .setup_reference_slot(&setup_slot)
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
            (functions.cmd_end_video_coding_khr)(command_buffer, &vk::VideoEndCodingInfoKHR::default());
        }
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

    fn read_feedback(
        memory: &MemoryContext,
        pool: &DeviceObject<vk::QueryPool>,
        input: usize,
    ) -> Result<EncodeFeedback, VideoError> {
        let mut results = [EncodeFeedback::default()];
        unsafe {
            memory.device().get_query_pool_results(
                pool.handle(),
                input as u32,
                &mut results,
                vk::QueryResultFlags::WITH_STATUS_KHR,
            )
        }?;
        let feedback = results[0];

        if feedback.status != vk::QueryResultStatusKHR::COMPLETE.as_raw() {
            return Err(VideoError::Encoding(feedback.status));
        }

        Ok(feedback)
    }
}
