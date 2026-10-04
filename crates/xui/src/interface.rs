use std::any::TypeId;
use std::collections::HashSet;

use crate::atlas::GlyphAtlas;
use crate::{
    Alignment, DrawList, Environment, FontLibrary, LayoutContext, Node, Point, PointerEvent, Rect, ScaleFactorKey,
    Size, SizeProposal, Subview, UpdateContext, View,
};

// Renders a view tree frame after frame, keeping its nodes in between: each render updates the nodes from the
// new views, then lays them out.
pub struct Interface {
    fonts: FontLibrary,
    builders: parley::LayoutContext<()>,
    atlas: GlyphAtlas,
    warnings: HashSet<String>,
    root: Option<Node>,
    scale_factor: f32,
}

impl Interface {
    pub fn new() -> Interface {
        Interface {
            fonts: FontLibrary::new(),
            builders: parley::LayoutContext::new(),
            atlas: GlyphAtlas::new(),
            warnings: HashSet::new(),
            root: None,
            scale_factor: 1.0,
        }
    }

    pub fn fonts_mut(&mut self) -> &mut FontLibrary {
        &mut self.fonts
    }

    // `viewport` is in physical pixels, like the returned draw list. The view is centered in it, as in SwiftUI;
    // frames position content anywhere else.
    pub fn render<Root: View>(
        &mut self,
        view: &Root,
        viewport: Size,
        environment: &Environment,
    ) -> anyhow::Result<DrawList> {
        let root = match &mut self.root {
            Some(root) if root.view_type() == TypeId::of::<Root>() => root,
            root => root.insert(Node::new(TypeId::of::<Root>(), None)),
        };
        let mut update = UpdateContext {
            environment: environment.clone(),
            fonts: &mut self.fonts,
            builders: &mut self.builders,
            warnings: &mut self.warnings,
            modifier_content: None,
        };
        view.update_node(root, &mut update)?;

        let scale_factor = environment.get::<ScaleFactorKey>();
        self.scale_factor = scale_factor;
        let container = Size {
            width: viewport.width / scale_factor,
            height: viewport.height / scale_factor,
        };
        let mut layout = LayoutContext {
            atlas: &mut self.atlas,
            stack_axis: None,
        };
        let size = root.size_that_fits(SizeProposal::from(container), &mut layout)?;
        let mut draw_list = DrawList::default();
        root.place(
            Rect {
                origin: Alignment::CENTER.position(container, size),
                size,
            },
            &mut layout,
            &mut draw_list,
        )?;
        draw_list.atlas_updates = self.atlas.take_updates();

        Ok(draw_list)
    }

    // Offers a pointer event to the views as they were last rendered, topmost first. Returns whether a view
    // used it, so the app can keep it from what lies under the interface.
    pub fn handle_event(&mut self, event: PointerEvent) -> bool {
        let Some(root) = &mut self.root else {
            return false;
        };
        let event = PointerEvent {
            position: Point {
                x: event.position.x / self.scale_factor,
                y: event.position.y / self.scale_factor,
            },
            ..event
        };

        root.handle_pointer(&event, false)
    }
}

impl Default for Interface {
    fn default() -> Interface {
        Interface::new()
    }
}
