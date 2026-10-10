use glam::Vec3;

use super::ObjectHandle;

pub struct Star {
    pub object: ObjectHandle,
    // Candela per color channel: illuminance at a distance d is intensity / d².
    pub intensity: Vec3,
}
