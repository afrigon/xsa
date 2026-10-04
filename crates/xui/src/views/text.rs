use crate::layout::TextLayout;
use crate::shaping::{ShapedText, TextInputs};
use crate::{Context, FontKey, ForegroundStyleKey, Never, Node, ScaleFactorKey, UpdateContext, View};

// Draws in the environment's font and foreground style; without a font it takes no space. Its node shapes the
// text again only when the string, font or scale factor changes.
pub struct Text {
    content: String,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Text {
        Text {
            content: content.into(),
        }
    }
}

impl View for Text {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let font = context.environment.get::<FontKey>();

        if font.is_none() {
            context.warn_once(format!("no font in the environment for the text {:?}", self.content));
        }

        let inputs = font.map(|font| TextInputs {
            content: self.content.clone(),
            font,
            scale_factor: context.environment.get::<ScaleFactorKey>(),
        });
        let color = context.environment.get::<ForegroundStyleKey>();
        let layout = node.layout_mut(TextLayout::new);
        layout.color = color;
        let reshape = layout.inputs != inputs;

        if reshape {
            layout.shaped = inputs
                .as_ref()
                .map(|inputs| ShapedText::shape(inputs, context.fonts, context.builders));
            layout.inputs = inputs;
            node.mark_changed();
        }

        node.update_children(&[], context)
    }
}
