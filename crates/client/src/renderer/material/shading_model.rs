#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShadingModel {
    Hapke,
    Lambert,
}

impl ShadingModel {
    // Matches the shading model constants in planet.slang.
    pub(in crate::renderer) fn shader_id(self) -> u32 {
        match self {
            ShadingModel::Hapke => 0,
            ShadingModel::Lambert => 1,
        }
    }
}
