#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tonemapper {
    Agx,
    AgxPunchy,
    KhronosNeutral,
}

// Matches the tonemapper constants in tonemap.slang.
pub(super) const NO_TONEMAPPER: u32 = 0;

impl Tonemapper {
    pub(super) fn shader_id(self) -> u32 {
        match self {
            Tonemapper::Agx => 1,
            Tonemapper::AgxPunchy => 2,
            Tonemapper::KhronosNeutral => 3,
        }
    }
}
