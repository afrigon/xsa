use std::collections::HashSet;

use parley::LayoutContext;

use crate::atlas::GlyphAtlas;
use crate::{Environment, FontLibrary};

pub struct ViewContext<'a> {
    pub environment: Environment,
    pub(crate) fonts: &'a mut FontLibrary,
    pub(crate) layouts: &'a mut LayoutContext<()>,
    pub(crate) atlas: &'a mut GlyphAtlas,
    pub(crate) warnings: &'a mut HashSet<String>,
}

impl ViewContext<'_> {
    pub fn with_environment(&mut self, environment: Environment) -> ViewContext<'_> {
        ViewContext {
            environment,
            fonts: self.fonts,
            layouts: self.layouts,
            atlas: self.atlas,
            warnings: self.warnings,
        }
    }

    // Layout runs every frame, so a problem is reported once rather than every frame.
    pub fn warn_once(&mut self, message: String) {
        if self.warnings.insert(message.clone()) {
            tracing::warn!("{message}");
        }
    }
}
