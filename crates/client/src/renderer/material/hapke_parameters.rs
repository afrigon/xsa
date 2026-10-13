use crate::renderer::TextureHandle;

// Hapke reflectance parameters of a particulate surface (regolith, frost). The scatter texture holds the
// single-scattering albedo w (red), the phase-function lobe width b (green) and lobe balance (c + 1) / 2 (blue); the
// surge texture holds the opposition-surge amplitudes and widths, B_S0 / 2, h_S, B_C0 / 2 and h_C.
#[derive(Clone, Copy)]
pub struct HapkeParameters {
    pub scatter: TextureHandle,
    pub surge: TextureHandle,
    // Hapke 2012's parameters other than the albedo, as stored: the particulate medium (b, c, θ̄ in degrees, porosity
    // K) and the opposition effects (B_S0, h_S, B_C0, h_C).
    pub medium: Option<TextureHandle>,
    pub opposition: Option<TextureHandle>,
    pub porosity: f32,
    pub roughness: f32,
    pub blend: f32,
    pub light_boost: f32,
    pub gamma_boost: f32,
}
