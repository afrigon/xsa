#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShadingModel {
    HapkeSol,
    HapkeTextbook,
    Lambert,
}

impl ShadingModel {
    // Matches the shading model constants in planet.slang.
    pub(in crate::renderer) fn shader_id(self) -> u32 {
        match self {
            ShadingModel::HapkeSol => 0,
            ShadingModel::HapkeTextbook => 1,
            ShadingModel::Lambert => 2,
        }
    }
}
