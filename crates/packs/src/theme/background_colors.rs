use crate::{Document, ParseContext};

use super::{Palette, ThemedColor};

pub struct BackgroundColors {
    pub default: ThemedColor,
    pub emphasis: ThemedColor,
    pub muted: ThemedColor,
    pub disabled: ThemedColor,
    pub inverse: ThemedColor,
}

impl BackgroundColors {
    pub fn parse(
        document: &Document,
        context: &ParseContext,
        palette: &mut Palette,
    ) -> anyhow::Result<BackgroundColors> {
        Ok(BackgroundColors {
            default: ThemedColor::from_document(document, "default", context, palette)?,
            emphasis: ThemedColor::from_document(document, "emphasis", context, palette)?,
            muted: ThemedColor::from_document(document, "muted", context, palette)?,
            disabled: ThemedColor::from_document(document, "disabled", context, palette)?,
            inverse: ThemedColor::from_document(document, "inverse", context, palette)?,
        })
    }
}
