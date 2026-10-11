// How the lens treated the frame's light sources.
#[derive(Clone, Copy, Default)]
pub struct LensLights {
    pub drawn: u32,
    // In front of the camera, but too faint for their lens effects to change a pixel.
    pub culled: u32,
    // Behind the camera or surrounding it.
    pub behind: u32,
}
