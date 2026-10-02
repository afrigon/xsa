#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tonemapper {
    Agx,
    AgxPunchy,
    KhronosNeutral,
    Off,
}

impl Tonemapper {
    pub fn next(self) -> Tonemapper {
        match self {
            Tonemapper::Agx => Tonemapper::AgxPunchy,
            Tonemapper::AgxPunchy => Tonemapper::KhronosNeutral,
            Tonemapper::KhronosNeutral => Tonemapper::Off,
            Tonemapper::Off => Tonemapper::Agx,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Tonemapper::Agx => "AgX",
            Tonemapper::AgxPunchy => "AgX Punchy",
            Tonemapper::KhronosNeutral => "Khronos PBR Neutral",
            Tonemapper::Off => "off (clipped)",
        }
    }

    // Matches the tonemapper constants in tonemap.slang.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            Tonemapper::Agx => 0,
            Tonemapper::AgxPunchy => 1,
            Tonemapper::KhronosNeutral => 2,
            Tonemapper::Off => 3,
        }
    }
}
