use std::collections::HashSet;

use parley::LayoutContext;

use crate::atlas::GlyphAtlas;
use crate::text_layouts::TextLayoutCache;
use crate::{
    Alignment, DrawList, Environment, FontLibrary, Rect, ScaleFactorKey, Size, SizeProposal, View, ViewContext,
};

pub struct Interface {
    fonts: FontLibrary,
    layouts: LayoutContext<()>,
    atlas: GlyphAtlas,
    text_layouts: TextLayoutCache,
    warnings: HashSet<String>,
}

impl Interface {
    pub fn new() -> Interface {
        Interface {
            fonts: FontLibrary::new(),
            layouts: LayoutContext::new(),
            atlas: GlyphAtlas::new(),
            text_layouts: TextLayoutCache::new(),
            warnings: HashSet::new(),
        }
    }

    pub fn fonts_mut(&mut self) -> &mut FontLibrary {
        &mut self.fonts
    }

    // `viewport` is in physical pixels, like the returned draw list. The view is centered in it, as in SwiftUI;
    // frames position content anywhere else.
    pub fn render(&mut self, view: &impl View, viewport: Size, environment: &Environment) -> anyhow::Result<DrawList> {
        let scale_factor = environment.get::<ScaleFactorKey>();
        let container = Size {
            width: viewport.width / scale_factor,
            height: viewport.height / scale_factor,
        };
        let mut context = ViewContext {
            environment: environment.clone(),
            fonts: &mut self.fonts,
            layouts: &mut self.layouts,
            atlas: &mut self.atlas,
            text_layouts: &mut self.text_layouts,
            warnings: &mut self.warnings,
            stack_axis: None,
        };
        let size = view.size_that_fits(SizeProposal::from(container), &mut context)?;
        let mut draw_list = DrawList::default();
        view.place(
            Rect {
                origin: Alignment::CENTER.position(container, size),
                size,
            },
            &mut context,
            &mut draw_list,
        )?;
        draw_list.atlas_updates = self.atlas.take_updates();
        self.text_layouts.end_frame();

        Ok(draw_list)
    }
}

impl Default for Interface {
    fn default() -> Interface {
        Interface::new()
    }
}
