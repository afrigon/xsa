use super::ConfigChoice;
use crate::renderer::SurfaceTiling;

impl ConfigChoice for SurfaceTiling {
    const ALL: &'static [Self] = &[SurfaceTiling::Hex, SurfaceTiling::Offset];

    fn config_name(self) -> &'static str {
        match self {
            SurfaceTiling::Hex => "hex",
            SurfaceTiling::Offset => "offset",
        }
    }
}
