use xsa_config::ConfigChoice;

use crate::config::AntialiasingKind;

impl ConfigChoice for AntialiasingKind {
    const ALL: &'static [Self] = &[AntialiasingKind::Taa];

    fn config_name(self) -> &'static str {
        match self {
            AntialiasingKind::Taa => "taa",
        }
    }
}
