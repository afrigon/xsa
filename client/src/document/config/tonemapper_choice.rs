use super::ConfigChoice;
use crate::renderer::Tonemapper;

impl ConfigChoice for Tonemapper {
    const ALL: &'static [Self] = &[
        Tonemapper::Agx,
        Tonemapper::AgxPunchy,
        Tonemapper::KhronosNeutral,
        Tonemapper::Off,
    ];

    fn config_name(self) -> &'static str {
        match self {
            Tonemapper::Agx => "agx",
            Tonemapper::AgxPunchy => "agx-punchy",
            Tonemapper::KhronosNeutral => "khronos-pbr-neutral",
            Tonemapper::Off => "off",
        }
    }
}
