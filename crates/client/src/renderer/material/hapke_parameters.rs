use resource_vulkan::{ResourceHandle, Texture};

// Hapke 2012's parameters of a particulate surface (regolith, frost) other than the albedo, which comes from the
// color map: the particulate medium (b, c, θ̄ in degrees, porosity K) and the opposition effects (B_S0, h_S, B_C0,
// h_C), stored as published.
#[derive(Clone)]
pub struct HapkeParameters {
    pub medium: ResourceHandle<Texture>,
    pub opposition: ResourceHandle<Texture>,
}
