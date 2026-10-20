use super::SURFACE_CLASSES;

// A biome's ground by its slope: flat, sloped and steep ground each have their surface.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SurfaceClass {
    Flat,
    Slope,
    Steep,
}

impl SurfaceClass {
    pub const ALL: [SurfaceClass; SURFACE_CLASSES] = [SurfaceClass::Flat, SurfaceClass::Slope, SurfaceClass::Steep];

    pub fn name(self) -> &'static str {
        match self {
            SurfaceClass::Flat => "flat",
            SurfaceClass::Slope => "slope",
            SurfaceClass::Steep => "steep",
        }
    }
}
