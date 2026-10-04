use parley::LayoutContext;

use crate::atlas::GlyphAtlas;
use crate::{Alignment, DrawList, Environment, FontLibrary, Rect, Size, SizeProposal, View, ViewContext};

pub struct Interface {
    fonts: FontLibrary,
    layouts: LayoutContext<()>,
    atlas: GlyphAtlas,
}

impl Interface {
    pub fn new() -> Interface {
        Interface {
            fonts: FontLibrary::new(),
            layouts: LayoutContext::new(),
            atlas: GlyphAtlas::new(),
        }
    }

    pub fn fonts_mut(&mut self) -> &mut FontLibrary {
        &mut self.fonts
    }

    // `viewport` is in physical pixels, like the returned draw list.
    pub fn render(
        &mut self,
        view: &impl View,
        viewport: Size,
        environment: &Environment,
        alignment: Alignment,
    ) -> anyhow::Result<DrawList> {
        let container = Size {
            width: viewport.width / environment.scale_factor,
            height: viewport.height / environment.scale_factor,
        };
        let mut context = ViewContext {
            environment,
            fonts: &mut self.fonts,
            layouts: &mut self.layouts,
            atlas: &mut self.atlas,
        };
        let size = view.size_that_fits(SizeProposal::from(container), &mut context)?;
        let mut draw_list = DrawList::default();
        view.place(
            Rect {
                origin: alignment.position(container, size),
                size,
            },
            &mut context,
            &mut draw_list,
        )?;
        draw_list.atlas_updates = self.atlas.take_updates();

        Ok(draw_list)
    }
}

impl Default for Interface {
    fn default() -> Interface {
        Interface::new()
    }
}
