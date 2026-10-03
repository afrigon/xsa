use crate::{Document, ParseContext};

use super::{Palette, ThemedColor};

pub struct ColorRole {
    pub emphasis: ThemedColor,
    pub muted: ThemedColor,
    pub foreground: ThemedColor,
    pub border_emphasis: ThemedColor,
    pub border_muted: ThemedColor,
}

impl ColorRole {
    pub fn parse(document: &Document, context: &ParseContext, palette: &mut Palette) -> anyhow::Result<ColorRole> {
        let emphasis = ThemedColor::from_document(document, "emphasis", context, palette)?;
        let muted = ThemedColor::from_document(document, "muted", context, palette)?;

        Ok(ColorRole {
            emphasis,
            muted,
            foreground: ThemedColor::optional_from_document(document, "foreground", context, palette)?
                .unwrap_or(emphasis),
            border_emphasis: ThemedColor::optional_from_document(document, "border-emphasis", context, palette)?
                .unwrap_or(emphasis),
            border_muted: ThemedColor::optional_from_document(document, "border-muted", context, palette)?
                .unwrap_or(muted),
        })
    }
}
