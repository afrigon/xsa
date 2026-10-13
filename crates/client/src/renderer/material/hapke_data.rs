use super::{HapkeParameters, NO_TEXTURE};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct HapkeData {
    pub medium_texture: u32,
    pub opposition_texture: u32,
}

impl HapkeData {
    pub const NONE: HapkeData = HapkeData {
        medium_texture: NO_TEXTURE,
        opposition_texture: NO_TEXTURE,
    };
}

impl From<HapkeParameters> for HapkeData {
    fn from(hapke: HapkeParameters) -> Self {
        HapkeData {
            medium_texture: hapke.medium.index(),
            opposition_texture: hapke.opposition.index(),
        }
    }
}
