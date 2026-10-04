use parley::LayoutContext;

use crate::atlas::GlyphAtlas;
use crate::{Environment, FontLibrary};

pub struct ViewContext<'a> {
    pub environment: &'a Environment,
    pub(crate) fonts: &'a mut FontLibrary,
    pub(crate) layouts: &'a mut LayoutContext<()>,
    pub(crate) atlas: &'a mut GlyphAtlas,
}
