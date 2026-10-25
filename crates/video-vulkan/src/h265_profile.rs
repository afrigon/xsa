use std::slice;

use ash::vk;
use ash::vk::native::StdVideoH265ProfileIdc_STD_VIDEO_H265_PROFILE_IDC_MAIN;

// H.265 Main: 8-bit 4:2:0 frames. Vulkan reads the profile through a chain of structures that point at each other, so
// it is built for the duration of each call that needs it.
pub(crate) struct H265Profile;

impl H265Profile {
    pub fn with<R>(f: impl FnOnce(&vk::VideoProfileInfoKHR) -> R) -> R {
        let mut h265 = vk::VideoEncodeH265ProfileInfoKHR::default()
            .std_profile_idc(StdVideoH265ProfileIdc_STD_VIDEO_H265_PROFILE_IDC_MAIN);
        let profile = vk::VideoProfileInfoKHR::default()
            .video_codec_operation(vk::VideoCodecOperationFlagsKHR::ENCODE_H265)
            .chroma_subsampling(vk::VideoChromaSubsamplingFlagsKHR::TYPE_420)
            .luma_bit_depth(vk::VideoComponentBitDepthFlagsKHR::TYPE_8)
            .chroma_bit_depth(vk::VideoComponentBitDepthFlagsKHR::TYPE_8)
            .push_next(&mut h265);

        f(&profile)
    }

    // Images, buffers and query pools a session uses name its profile in a list.
    pub fn with_list<R>(f: impl FnOnce(&mut vk::VideoProfileListInfoKHR) -> R) -> R {
        H265Profile::with(|profile| {
            let mut list = vk::VideoProfileListInfoKHR::default().profiles(slice::from_ref(profile));

            f(&mut list)
        })
    }
}
