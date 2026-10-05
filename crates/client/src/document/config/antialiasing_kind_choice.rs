use super::ConfigChoice;
use crate::config::AntialiasingKind;

impl ConfigChoice for Option<AntialiasingKind> {
    const ALL: &'static [Self] = &[None, Some(AntialiasingKind::Taa)];

    fn config_name(self) -> &'static str {
        match self {
            None => "none",
            Some(AntialiasingKind::Taa) => "taa",
        }
    }
}
