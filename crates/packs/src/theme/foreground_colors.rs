use crate::{Document, ParseContext};

use super::{Palette, ThemedColor};

pub struct ForegroundColors {
    pub default: ThemedColor,
    pub muted: ThemedColor,
    pub on_emphasis: ThemedColor,
    pub disabled: ThemedColor,
    pub link: ThemedColor,
}

impl ForegroundColors {
    pub fn parse(
        document: &Document,
        context: &ParseContext,
        palette: &mut Palette,
    ) -> anyhow::Result<ForegroundColors> {
        Ok(ForegroundColors {
            default: ThemedColor::from_document(document, "default", context, palette)?,
            muted: ThemedColor::from_document(document, "muted", context, palette)?,
            on_emphasis: ThemedColor::from_document(document, "on-emphasis", context, palette)?,
            disabled: ThemedColor::from_document(document, "disabled", context, palette)?,
            link: ThemedColor::from_document(document, "link", context, palette)?,
        })
    }
}
