use super::ConfigChoice;
use crate::renderer::ShadingModel;

impl ConfigChoice for ShadingModel {
    const ALL: &'static [Self] = &[
        ShadingModel::HapkeSol,
        ShadingModel::HapkeTextbook,
        ShadingModel::HapkeClean,
        ShadingModel::Lambert,
    ];

    fn config_name(self) -> &'static str {
        match self {
            ShadingModel::HapkeSol => "hapke-sol",
            ShadingModel::HapkeTextbook => "hapke-textbook",
            ShadingModel::Lambert => "lambert",
            ShadingModel::HapkeClean => "hapke-clean",
        }
    }
}
