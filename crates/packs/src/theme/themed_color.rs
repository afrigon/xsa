use anyhow::{Context, ensure};
use kdl::KdlNode;

use crate::{Document, FromNode, NodeExtension, ParseContext};

use super::{ColorReference, Palette, SrgbColor};

const LIGHT: &str = "light";
const DARK: &str = "dark";
const LIGHT_INCREASED_CONTRAST: &str = "light-increased-contrast";
const DARK_INCREASED_CONTRAST: &str = "dark-increased-contrast";

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ThemedColor {
    pub light: SrgbColor,
    pub dark: SrgbColor,
    pub light_increased_contrast: SrgbColor,
    pub dark_increased_contrast: SrgbColor,
}

impl ThemedColor {
    pub fn from_document(
        document: &Document,
        name: &str,
        context: &ParseContext,
        palette: &mut Palette,
    ) -> anyhow::Result<ThemedColor> {
        ThemedColor::parse(document.node(name)?, context, palette)
    }

    pub fn optional_from_document(
        document: &Document,
        name: &str,
        context: &ParseContext,
        palette: &mut Palette,
    ) -> anyhow::Result<Option<ThemedColor>> {
        document
            .optional_node(name)
            .map(|node| ThemedColor::parse(node, context, palette))
            .transpose()
    }

    pub fn parse(node: &KdlNode, context: &ParseContext, palette: &mut Palette) -> anyhow::Result<ThemedColor> {
        let children = node.child_nodes();

        if children.is_empty() {
            let light = ColorReference::from_node(node, context)?;
            return ThemedColor::derive(light, None, None, None, palette);
        }

        ensure!(
            node.arguments().next().is_none(),
            "`{}` takes either a color or a block of scheme variants, not both",
            node.node_name()
        );
        let variant = |name: &str| -> anyhow::Result<Option<ColorReference>> {
            children
                .iter()
                .find(|child| child.node_name() == name)
                .map(|child| ColorReference::from_node(child, context))
                .transpose()
        };

        for child in children {
            ensure!(
                [LIGHT, DARK, LIGHT_INCREASED_CONTRAST, DARK_INCREASED_CONTRAST].contains(&child.node_name()),
                "unknown scheme variant `{}`: use {LIGHT}, {DARK}, {LIGHT_INCREASED_CONTRAST} or {DARK_INCREASED_CONTRAST}",
                child.node_name()
            );
        }

        let light = variant(LIGHT)?.with_context(|| format!("`{}` needs a `{LIGHT}` variant", node.node_name()))?;

        ThemedColor::derive(
            light,
            variant(DARK)?,
            variant(LIGHT_INCREASED_CONTRAST)?,
            variant(DARK_INCREASED_CONTRAST)?,
            palette,
        )
    }

    fn derive(
        light: ColorReference,
        dark: Option<ColorReference>,
        light_increased_contrast: Option<ColorReference>,
        dark_increased_contrast: Option<ColorReference>,
        palette: &mut Palette,
    ) -> anyhow::Result<ThemedColor> {
        let dark = dark.unwrap_or_else(|| light.with_tint(light.tint.inverse()));
        let light_increased_contrast = light_increased_contrast.unwrap_or_else(|| light.with_tint(light.tint.lower()));
        let dark_increased_contrast = dark_increased_contrast.unwrap_or_else(|| dark.with_tint(dark.tint.higher()));

        Ok(ThemedColor {
            light: palette.resolve(&light)?,
            dark: palette.resolve(&dark)?,
            light_increased_contrast: palette.resolve(&light_increased_contrast)?,
            dark_increased_contrast: palette.resolve(&dark_increased_contrast)?,
        })
    }
}
