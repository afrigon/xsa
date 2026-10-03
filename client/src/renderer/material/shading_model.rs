#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShadingModel {
    HapkeSol,
    HapkeTextbook,
    Lambert,
}

impl ShadingModel {
    pub fn next(self) -> ShadingModel {
        match self {
            ShadingModel::HapkeSol => ShadingModel::HapkeTextbook,
            ShadingModel::HapkeTextbook => ShadingModel::Lambert,
            ShadingModel::Lambert => ShadingModel::HapkeSol,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ShadingModel::HapkeSol => "Hapke (Sol)",
            ShadingModel::HapkeTextbook => "Hapke (textbook)",
            ShadingModel::Lambert => "Lambert",
        }
    }

    // Matches the shading model constants in planet.slang.
    pub(in crate::renderer) fn shader_id(self) -> u32 {
        match self {
            ShadingModel::HapkeSol => 0,
            ShadingModel::HapkeTextbook => 1,
            ShadingModel::Lambert => 2,
        }
    }
}
