use super::ConfigChoice;
use crate::renderer::Tonemapper;

impl ConfigChoice for Option<Tonemapper> {
    const ALL: &'static [Self] = &[
        None,
        Some(Tonemapper::Agx),
        Some(Tonemapper::AgxPunchy),
        Some(Tonemapper::KhronosNeutral),
    ];

    fn config_name(self) -> &'static str {
        match self {
            None => "none",
            Some(Tonemapper::Agx) => "agx",
            Some(Tonemapper::AgxPunchy) => "agx-punchy",
            Some(Tonemapper::KhronosNeutral) => "khronos-pbr-neutral",
        }
    }
}
