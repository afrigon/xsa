use std::ptr;

use ash::khr::video_queue;
use ash::vk;
use memory_vulkan::MemoryContext;

use crate::VideoError;
use crate::h265_profile::H265Profile;

const PICTURE_FORMAT: vk::Format = vk::Format::G8_B8R8_2PLANE_420_UNORM;

// What the GPU's H.265 encoder supports, as far as the encoder relies on it.
pub(crate) struct EncoderCapabilities {
    pub bitstream_size_alignment: u64,
    pub picture_granularity: vk::Extent2D,
    pub max_coded_extent: vk::Extent2D,
    pub std_header_version: vk::ExtensionProperties,
    pub ctb_sizes: vk::VideoEncodeH265CtbSizeFlagsKHR,
    pub transform_block_sizes: vk::VideoEncodeH265TransformBlockSizeFlagsKHR,
    pub min_qp: i32,
    pub max_qp: i32,
    pub max_quality_levels: u32,
}

impl EncoderCapabilities {
    pub fn query(entry: &ash::Entry, memory: &MemoryContext) -> Result<EncoderCapabilities, VideoError> {
        let functions = video_queue::Instance::new(entry, memory.instance());
        let physical_device = memory.physical_device();
        let mut h265 = vk::VideoEncodeH265CapabilitiesKHR::default();
        let mut encode = vk::VideoEncodeCapabilitiesKHR::default();
        let mut capabilities = vk::VideoCapabilitiesKHR::default()
            .push_next(&mut encode)
            .push_next(&mut h265);
        H265Profile::with(|profile| unsafe {
            (functions.fp().get_physical_device_video_capabilities_khr)(physical_device, profile, &mut capabilities)
        })
        .result()?;
        let bitstream_size_alignment = capabilities.min_bitstream_buffer_size_alignment;
        let access_granularity = capabilities.picture_access_granularity;
        let max_coded_extent = capabilities.max_coded_extent;
        let std_header_version = capabilities.std_header_version;
        let flags = capabilities.flags;

        let wanted_feedback = vk::VideoEncodeFeedbackFlagsKHR::BITSTREAM_BUFFER_OFFSET
            | vk::VideoEncodeFeedbackFlagsKHR::BITSTREAM_BYTES_WRITTEN;

        if !encode
            .rate_control_modes
            .contains(vk::VideoEncodeRateControlModeFlagsKHR::DISABLED)
        {
            return Err(VideoError::Unsupported("constant-quality encoding".to_string()));
        }

        if !encode.supported_encode_feedback_flags.contains(wanted_feedback) {
            return Err(VideoError::Unsupported("bitstream feedback".to_string()));
        }

        if !flags.contains(vk::VideoCapabilityFlagsKHR::SEPARATE_REFERENCE_IMAGES) {
            return Err(VideoError::Unsupported(
                "reference pictures in separate images".to_string(),
            ));
        }

        if h265.max_p_picture_l0_reference_count == 0 {
            return Err(VideoError::Unsupported("predicted frames".to_string()));
        }

        for usage in [
            vk::ImageUsageFlags::VIDEO_ENCODE_SRC_KHR,
            vk::ImageUsageFlags::VIDEO_ENCODE_DPB_KHR,
        ] {
            if !EncoderCapabilities::formats(&functions, physical_device, usage)?.contains(&PICTURE_FORMAT) {
                return Err(VideoError::Unsupported(format!("NV12 pictures for {usage:?}")));
            }
        }

        Ok(EncoderCapabilities {
            bitstream_size_alignment,
            picture_granularity: vk::Extent2D {
                width: access_granularity
                    .width
                    .max(encode.encode_input_picture_granularity.width),
                height: access_granularity
                    .height
                    .max(encode.encode_input_picture_granularity.height),
            },
            max_coded_extent,
            std_header_version,
            ctb_sizes: h265.ctb_sizes,
            transform_block_sizes: h265.transform_block_sizes,
            min_qp: h265.min_qp,
            max_qp: h265.max_qp,
            max_quality_levels: encode.max_quality_levels,
        })
    }

    pub fn picture_format(&self) -> vk::Format {
        PICTURE_FORMAT
    }

    fn formats(
        functions: &video_queue::Instance,
        physical_device: vk::PhysicalDevice,
        usage: vk::ImageUsageFlags,
    ) -> Result<Vec<vk::Format>, VideoError> {
        H265Profile::with_list(|list| {
            let info = vk::PhysicalDeviceVideoFormatInfoKHR::default()
                .image_usage(usage)
                .push_next(list);
            let query = functions.fp().get_physical_device_video_format_properties_khr;
            let mut count = 0;
            unsafe { query(physical_device, &info, &mut count, ptr::null_mut()) }.result()?;
            let mut properties = vec![vk::VideoFormatPropertiesKHR::default(); count as usize];
            unsafe { query(physical_device, &info, &mut count, properties.as_mut_ptr()) }.result()?;

            Ok(properties.iter().map(|property| property.format).collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use memory_vulkan::{TestDevice, TestRequirements};

    use super::*;
    use crate::VideoEncoder;

    #[test]
    fn the_gpu_reports_what_the_encoder_relies_on() {
        let requirements = TestRequirements {
            extensions: VideoEncoder::DEVICE_EXTENSIONS.to_vec(),
            queue: Some(VideoEncoder::QUEUE_CAPABILITY),
        };
        let Ok(device) = TestDevice::with(&requirements) else {
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
}
