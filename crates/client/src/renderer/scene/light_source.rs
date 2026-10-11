use glam::Vec3;

use super::ObjectHandle;

// Anything the camera's lens responds to: the light comes from the object's position.
pub struct LightSource {
    pub object: ObjectHandle,
    // Candela per color channel: illuminance at a distance d is intensity / d².
    pub intensity: Vec3,
}
