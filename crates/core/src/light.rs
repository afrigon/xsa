mod disk_occlusion;
mod illumination;
mod star_light;

pub use disk_occlusion::visible_disk_fraction;
pub use illumination::Illumination;
pub use star_light::StarLight;

// Light from the stars of the sky and zodiacal light, in lux.
pub const BACKGROUND_ILLUMINANCE: f64 = 2e-4;
