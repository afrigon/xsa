use std::{mem, ptr};

use ash::khr::{video_encode_queue, video_queue};
use ash::vk;
use ash::vk::native::{
    StdVideoH265ChromaFormatIdc_STD_VIDEO_H265_CHROMA_FORMAT_IDC_420, StdVideoH265DecPicBufMgr, StdVideoH265LevelIdc,
    StdVideoH265LevelIdc_STD_VIDEO_H265_LEVEL_IDC_4_1, StdVideoH265LevelIdc_STD_VIDEO_H265_LEVEL_IDC_5_1,
    StdVideoH265LevelIdc_STD_VIDEO_H265_LEVEL_IDC_6_2, StdVideoH265PictureParameterSet,
    StdVideoH265ProfileIdc_STD_VIDEO_H265_PROFILE_IDC_MAIN, StdVideoH265ProfileTierLevel,
    StdVideoH265SequenceParameterSet, StdVideoH265SequenceParameterSetVui, StdVideoH265VideoParameterSet,
};
use gpu_allocator::vulkan::Allocation;
use memory_vulkan::{DeviceObject, MemoryContext, MemoryLocation};

use crate::VideoError;
use crate::encoder_capabilities::EncoderCapabilities;
use crate::h265_profile::H265Profile;
use crate::picture_geometry::PictureGeometry;
use crate::video_session::VideoSession;
use crate::video_session_parameters::VideoSessionParameters;

// Pictures the decoder holds at once: the one being decoded and the one it references.
const DPB_SLOTS: u32 = 2;
const MAX_POC_LSB_LOG2: u8 = 8;
// H.265 Table A.8: the largest picture and the most luma samples per second each level allows.
const LEVEL_4_1_PICTURE_SAMPLES: u64 = 2_228_224;
const LEVEL_4_1_SAMPLE_RATE: u64 = 133_693_440;
const LEVEL_5_1_PICTURE_SAMPLES: u64 = 8_912_896;
const LEVEL_5_1_SAMPLE_RATE: u64 = 534_773_760;
// H.273 code points: BT.709 primaries, transfer and matrix; video_format 5 is unspecified.
const BT_709: u8 = 1;
const UNSPECIFIED_VIDEO_FORMAT: u8 = 5;

// The encode session, its memory and its parameter sets, with the stream headers they encode to.
pub(crate) struct EncodeSession {
    pub functions: video_queue::Device,
    pub encode_functions: video_encode_queue::Device,
    // Dropped in this order: the parameters, the session, then the memory bound to it.
    pub parameters: DeviceObject<VideoSessionParameters>,
    pub session: DeviceObject<VideoSession>,
    _memory: Vec<DeviceObject<Allocation>>,
    pub quality_level: u32,
    pub headers: Vec<u8>,
}

impl EncodeSession {
    pub fn new(
        memory: &MemoryContext,
        capabilities: &EncoderCapabilities,
        queue_family: u32,
        geometry: &PictureGeometry,
        fps: u32,
    ) -> Result<EncodeSession, VideoError> {
        let device = memory.device();
        let functions = video_queue::Device::new(memory.instance(), device);
        let encode_functions = video_encode_queue::Device::new(memory.instance(), device);
        let format = capabilities.picture_format();
        let handle = H265Profile::with(|profile| {
            let info = vk::VideoSessionCreateInfoKHR::default()
                .queue_family_index(queue_family)
                .video_profile(profile)
                .picture_format(format)
                .max_coded_extent(geometry.coded_extent)
                .reference_picture_format(format)
                .max_dpb_slots(DPB_SLOTS)
                .max_active_reference_pictures(1)
                .std_header_version(&capabilities.std_header_version);
            let mut handle = vk::VideoSessionKHR::null();
            unsafe { (functions.fp().create_video_session_khr)(device.handle(), &info, ptr::null(), &mut handle) }
                .result()?;

            Ok::<_, VideoError>(handle)
        })?;
        let session = DeviceObject::new(
            memory,
            VideoSession {
                handle,
                functions: functions.clone(),
            },
        );
        let session_memory = EncodeSession::bind_memory(memory, &functions, handle)?;
        let quality_level = capabilities.max_quality_levels.saturating_sub(1);
        let parameters = EncodeSession::create_parameters(memory, &functions, handle, geometry, fps, quality_level)?;
        let headers = EncodeSession::encoded_headers(device, &encode_functions, parameters.get().handle)?;

        Ok(EncodeSession {
            functions,
            encode_functions,
            parameters,
            session,
            _memory: session_memory,
            quality_level,
            headers,
        })
    }

    fn bind_memory(
        memory: &MemoryContext,
        functions: &video_queue::Device,
        session: vk::VideoSessionKHR,
    ) -> Result<Vec<DeviceObject<Allocation>>, VideoError> {
        let device = memory.device().handle();
        let query = functions.fp().get_video_session_memory_requirements_khr;
        let mut count = 0;
        unsafe { query(device, session, &mut count, ptr::null_mut()) }.result()?;
        let mut requirements = vec![vk::VideoSessionMemoryRequirementsKHR::default(); count as usize];
        unsafe { query(device, session, &mut count, requirements.as_mut_ptr()) }.result()?;
        let mut allocations = Vec::with_capacity(requirements.len());
        let mut bindings = Vec::with_capacity(requirements.len());

        for requirement in &requirements {
            // The session names the memory types it accepts, which may exclude device-local ones.
            let allocation = memory.allocate_memory(
                "video session",
                requirement.memory_requirements,
                MemoryLocation::Unknown,
            )?;
            bindings.push(
                vk::BindVideoSessionMemoryInfoKHR::default()
                    .memory_bind_index(requirement.memory_bind_index)
                    .memory(unsafe { allocation.get().memory() })
                    .memory_offset(allocation.get().offset())
                    .memory_size(requirement.memory_requirements.size),
            );
            allocations.push(allocation);
        }

        unsafe {
            (functions.fp().bind_video_session_memory_khr)(device, session, bindings.len() as u32, bindings.as_ptr())
        }
        .result()?;

        Ok(allocations)
    }

    fn create_parameters(
        memory: &MemoryContext,
        functions: &video_queue::Device,
        session: vk::VideoSessionKHR,
        geometry: &PictureGeometry,
        fps: u32,
        quality_level: u32,
    ) -> Result<DeviceObject<VideoSessionParameters>, VideoError> {
        // The Std structures are plain C data without defaults; zero is a valid value for every field.
        let mut profile_tier_level: StdVideoH265ProfileTierLevel = unsafe { mem::zeroed() };
        profile_tier_level.general_profile_idc = StdVideoH265ProfileIdc_STD_VIDEO_H265_PROFILE_IDC_MAIN;
        profile_tier_level.general_level_idc = EncodeSession::level(geometry, fps);
        profile_tier_level.flags.set_general_progressive_source_flag(1);
        profile_tier_level.flags.set_general_frame_only_constraint_flag(1);
        let mut buffering: StdVideoH265DecPicBufMgr = unsafe { mem::zeroed() };
        buffering.max_dec_pic_buffering_minus1[0] = (DPB_SLOTS - 1) as u8;

        let mut video: StdVideoH265VideoParameterSet = unsafe { mem::zeroed() };
        video.flags.set_vps_temporal_id_nesting_flag(1);
        video.pDecPicBufMgr = &buffering;
        video.pProfileTierLevel = &profile_tier_level;

        let mut usability: StdVideoH265SequenceParameterSetVui = unsafe { mem::zeroed() };
        usability.flags.set_video_signal_type_present_flag(1);
        usability.flags.set_colour_description_present_flag(1);
        usability.flags.set_vui_timing_info_present_flag(1);
        usability.video_format = UNSPECIFIED_VIDEO_FORMAT;
        usability.colour_primaries = BT_709;
        usability.transfer_characteristics = BT_709;
        usability.matrix_coeffs = BT_709;
        usability.vui_num_units_in_tick = 1;
        usability.vui_time_scale = fps;

        let transform_depth = (geometry.ctb_log2 - geometry.min_transform_log2) as u8;
        let mut sequence: StdVideoH265SequenceParameterSet = unsafe { mem::zeroed() };
        sequence.flags.set_sps_temporal_id_nesting_flag(1);
        sequence.flags.set_vui_parameters_present_flag(1);
        sequence.chroma_format_idc = StdVideoH265ChromaFormatIdc_STD_VIDEO_H265_CHROMA_FORMAT_IDC_420;
        sequence.pic_width_in_luma_samples = geometry.coded_extent.width;
        sequence.pic_height_in_luma_samples = geometry.coded_extent.height;
        sequence.log2_max_pic_order_cnt_lsb_minus4 = MAX_POC_LSB_LOG2 - 4;
        sequence.log2_min_luma_coding_block_size_minus3 = (geometry.min_coding_block_log2() - 3) as u8;
        sequence.log2_diff_max_min_luma_coding_block_size =
            (geometry.ctb_log2 - geometry.min_coding_block_log2()) as u8;
        sequence.log2_min_luma_transform_block_size_minus2 = (geometry.min_transform_log2 - 2) as u8;
        sequence.log2_diff_max_min_luma_transform_block_size =
            (geometry.max_transform_log2 - geometry.min_transform_log2) as u8;
        sequence.max_transform_hierarchy_depth_inter = transform_depth;
        sequence.max_transform_hierarchy_depth_intra = transform_depth;
        sequence.pProfileTierLevel = &profile_tier_level;
        sequence.pDecPicBufMgr = &buffering;
        sequence.pSequenceParameterSetVui = &usability;

        // Offsets count chroma samples: 2 pixels in 4:2:0.
        let crop_right = (geometry.coded_extent.width - geometry.extent.width) / 2;
        let crop_bottom = (geometry.coded_extent.height - geometry.extent.height) / 2;

        if crop_right > 0 || crop_bottom > 0 {
            sequence.flags.set_conformance_window_flag(1);
            sequence.conf_win_right_offset = crop_right;
            sequence.conf_win_bottom_offset = crop_bottom;
        }

        let picture: StdVideoH265PictureParameterSet = unsafe { mem::zeroed() };
        let videos = [video];
        let sequences = [sequence];
        let pictures = [picture];
        let add_info = vk::VideoEncodeH265SessionParametersAddInfoKHR::default()
            .std_vp_ss(&videos)
            .std_sp_ss(&sequences)
            .std_pp_ss(&pictures);
        let mut h265 = vk::VideoEncodeH265SessionParametersCreateInfoKHR::default()
            .max_std_vps_count(1)
            .max_std_sps_count(1)
            .max_std_pps_count(1)
            .parameters_add_info(&add_info);
        let mut quality = vk::VideoEncodeQualityLevelInfoKHR::default().quality_level(quality_level);
        let info = vk::VideoSessionParametersCreateInfoKHR::default()
            .video_session(session)
            .push_next(&mut h265)
            .push_next(&mut quality);
        let mut handle = vk::VideoSessionParametersKHR::null();
        unsafe {
            (functions.fp().create_video_session_parameters_khr)(
                memory.device().handle(),
                &info,
                ptr::null(),
                &mut handle,
            )
        }
        .result()?;

        Ok(DeviceObject::new(
            memory,
            VideoSessionParameters {
                handle,
                functions: functions.clone(),
            },
        ))
    }

    // The VPS, SPS and PPS as the encoder writes them, which may differ from the ones given where it overrides them.
    fn encoded_headers(
        device: &ash::Device,
        functions: &video_encode_queue::Device,
        parameters: vk::VideoSessionParametersKHR,
    ) -> Result<Vec<u8>, VideoError> {
        let mut h265 = vk::VideoEncodeH265SessionParametersGetInfoKHR::default()
            .write_std_vps(true)
            .write_std_sps(true)
            .write_std_pps(true);
        let info = vk::VideoEncodeSessionParametersGetInfoKHR::default()
            .video_session_parameters(parameters)
            .push_next(&mut h265);
        let query = functions.fp().get_encoded_video_session_parameters_khr;
        let mut size = 0;
        unsafe { query(device.handle(), &info, ptr::null_mut(), &mut size, ptr::null_mut()) }.result()?;
        let mut headers = vec![0; size];
        unsafe {
            query(
                device.handle(),
                &info,
                ptr::null_mut(),
                &mut size,
                headers.as_mut_ptr().cast(),
            )
        }
        .result()?;
        headers.truncate(size);

        Ok(headers)
    }

    fn level(geometry: &PictureGeometry, fps: u32) -> StdVideoH265LevelIdc {
        let picture_samples = u64::from(geometry.coded_extent.width) * u64::from(geometry.coded_extent.height);
        let sample_rate = picture_samples * u64::from(fps);

        if picture_samples <= LEVEL_4_1_PICTURE_SAMPLES && sample_rate <= LEVEL_4_1_SAMPLE_RATE {
            StdVideoH265LevelIdc_STD_VIDEO_H265_LEVEL_IDC_4_1
        } else if picture_samples <= LEVEL_5_1_PICTURE_SAMPLES && sample_rate <= LEVEL_5_1_SAMPLE_RATE {
            StdVideoH265LevelIdc_STD_VIDEO_H265_LEVEL_IDC_5_1
        } else {
            StdVideoH265LevelIdc_STD_VIDEO_H265_LEVEL_IDC_6_2
        }
    }
}

#[cfg(test)]
mod tests {
    use memory_vulkan::{TestDevice, TestRequirements};

    use super::*;
    use crate::VideoEncoder;

    #[test]
    fn a_session_encodes_its_stream_headers() {
        let requirements = TestRequirements {
            extensions: VideoEncoder::DEVICE_EXTENSIONS.to_vec(),
            queue: Some(VideoEncoder::QUEUE_CAPABILITY),
        };
        let Ok(device) = TestDevice::with(&requirements) else {
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
}
