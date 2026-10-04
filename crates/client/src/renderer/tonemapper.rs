#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tonemapper {
    Agx,
    AgxPunchy,
    KhronosNeutral,
    Off,
}

impl Tonemapper {
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
