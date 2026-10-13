use super::ConfigChoice;
use crate::renderer::ShadingModel;

impl ConfigChoice for ShadingModel {
    const ALL: &'static [Self] = &[ShadingModel::Hapke, ShadingModel::Lambert];

    fn config_name(self) -> &'static str {
        match self {
            ShadingModel::Hapke => "hapke",
            ShadingModel::Lambert => "lambert",
        }
    }
}
