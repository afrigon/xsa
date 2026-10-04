use std::collections::HashSet;

use crate::{Environment, FontLibrary, Subview};

// What the update pass carries down the tree: the environment, what text shaping needs, and the content a
// view modifier wraps.
pub struct UpdateContext<'a> {
    pub environment: Environment,
    pub(crate) fonts: &'a mut FontLibrary,
    pub(crate) builders: &'a mut parley::LayoutContext<()>,
    pub(crate) warnings: &'a mut HashSet<String>,
    pub(crate) modifier_content: Option<&'a dyn Subview>,
}

impl UpdateContext<'_> {
    pub fn with_environment(&mut self, environment: Environment) -> UpdateContext<'_> {
        UpdateContext {
            environment,
            fonts: self.fonts,
            builders: self.builders,
            warnings: self.warnings,
            modifier_content: self.modifier_content,
        }
    }

    pub(crate) fn with_modifier_content<'b>(&'b mut self, content: Option<&'b dyn Subview>) -> UpdateContext<'b> {
        UpdateContext {
            environment: self.environment.clone(),
            fonts: self.fonts,
            builders: self.builders,
            warnings: self.warnings,
            modifier_content: content,
        }
    }

    // Updates run every frame, so a problem is reported once rather than every frame.
    pub fn warn_once(&mut self, message: String) {
        if self.warnings.insert(message.clone()) {
            tracing::warn!("{message}");
        }
    }
}
