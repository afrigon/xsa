use std::collections::HashSet;

use parley::LayoutContext;

use crate::atlas::GlyphAtlas;
use crate::layout::Axis;
use crate::text_layouts::TextLayoutCache;
use crate::{Environment, FontLibrary};

pub struct ViewContext<'a> {
    pub environment: Environment,
    pub(crate) fonts: &'a mut FontLibrary,
    pub(crate) layouts: &'a mut LayoutContext<()>,
    pub(crate) atlas: &'a mut GlyphAtlas,
    pub(crate) text_layouts: &'a mut TextLayoutCache,
    pub(crate) warnings: &'a mut HashSet<String>,
    pub(crate) stack_axis: Option<Axis>,
}

impl ViewContext<'_> {
    pub fn with_environment(&mut self, environment: Environment) -> ViewContext<'_> {
        ViewContext {
            environment,
            fonts: self.fonts,
            layouts: self.layouts,
            atlas: self.atlas,
            text_layouts: self.text_layouts,
            warnings: self.warnings,
            stack_axis: self.stack_axis,
        }
    }

    pub(crate) fn with_stack_axis(&mut self, stack_axis: Option<Axis>) -> ViewContext<'_> {
        ViewContext {
            environment: self.environment.clone(),
            fonts: self.fonts,
            layouts: self.layouts,
            atlas: self.atlas,
            text_layouts: self.text_layouts,
            warnings: self.warnings,
            stack_axis,
        }
    }

    // Layout runs every frame, so a problem is reported once rather than every frame.
    pub fn warn_once(&mut self, message: String) {
        if self.warnings.insert(message.clone()) {
            tracing::warn!("{message}");
        }
    }
}
