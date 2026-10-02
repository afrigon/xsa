#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tonemapper {
    Agx,
    KhronosNeutral,
    Clip,
}

impl Tonemapper {
    pub fn next(self) -> Tonemapper {
        match self {
            Tonemapper::Agx => Tonemapper::KhronosNeutral,
            Tonemapper::KhronosNeutral => Tonemapper::Clip,
            Tonemapper::Clip => Tonemapper::Agx,
        }
    }

    // Matches the tonemapper constants in tonemap.slang.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            Tonemapper::Agx => 0,
            Tonemapper::KhronosNeutral => 1,
            Tonemapper::Clip => 2,
        }
    }
}
