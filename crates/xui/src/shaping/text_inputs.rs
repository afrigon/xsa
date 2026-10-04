use crate::Font;

// Everything that changes how a text is shaped; a Text node shapes again only when these change.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct TextInputs {
    pub content: String,
    pub font: Font,
    pub scale_factor: f32,
}
