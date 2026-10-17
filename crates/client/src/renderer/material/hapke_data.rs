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

impl HapkeData {
    // The shader reads both maps once it has the medium, so none until both are ready.
    pub fn of(hapke: Option<&HapkeParameters>) -> HapkeData {
        let indices = hapke.and_then(|hapke| Some((hapke.medium.get()?.index(), hapke.opposition.get()?.index())));

        match indices {
            Some((medium_texture, opposition_texture)) => HapkeData {
                medium_texture,
                opposition_texture,
            },
            None => HapkeData::NONE,
        }
    }
}
