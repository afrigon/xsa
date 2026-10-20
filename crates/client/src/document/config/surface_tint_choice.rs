use super::ConfigChoice;
use crate::renderer::SurfaceTint;

impl ConfigChoice for SurfaceTint {
    const ALL: &'static [Self] = &[SurfaceTint::Relative, SurfaceTint::AsIs, SurfaceTint::Replace];

    fn config_name(self) -> &'static str {
        match self {
            SurfaceTint::Relative => "relative",
            SurfaceTint::AsIs => "as-is",
            SurfaceTint::Replace => "replace",
        }
    }
}
