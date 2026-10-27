use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterInstanceData {
    // From the page's center, in the face's coordinates.
    pub offset: [f32; 2],
    // The yaw (radians) in the low half and the scale in the high half, as half floats.
    pub yaw_and_scale: u32,
    // The lean from upright in the low half and the direction it leans toward in the high half, radians as half floats.
    pub tilt_and_heading: u32,
    // The share of the model's height below the ground.
    pub sink: f32,
    // Its model's place in the scatter's list.
    pub model: u32,
}

const _: () = assert!(size_of::<ScatterInstanceData>() == 24);

unsafe impl GpuData for ScatterInstanceData {}
