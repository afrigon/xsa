mod bloom_push_constants;
mod frame_data;
mod histogram_push_constants;
mod object_data;
mod push_constants;

pub(super) use bloom_push_constants::BloomPushConstants;
pub(super) use frame_data::FrameData;
pub(super) use histogram_push_constants::HistogramPushConstants;
pub(super) use object_data::ObjectData;
pub(super) use push_constants::PushConstants;

/// # Safety
///
/// Implementors are `#[repr(C)]` with no implicit padding, so every byte is initialized.
pub(super) unsafe trait GpuData: Copy {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const Self).cast::<u8>(), size_of::<Self>()) }
    }
}
