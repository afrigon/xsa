use std::collections::HashMap;

use anyhow::{Context, ensure};

use crate::{DOCUMENT_EXTENSION, Document, Id, PackStack, ParseContext};

use super::{BackgroundColors, BorderColors, ColorRole, ForegroundColors, Palette, TextStyle, Typeface};

const THEME_NAMESPACE: &str = "base";
const COLOR_ROLES: &str = "theme/color-roles";
const SURFACES: &str = "theme/surfaces";
const TYPEFACES: &str = "theme/typefaces";
const TEXT_STYLES: &str = "theme/text-styles";

pub struct ThemeDefinition {
    pub color_roles: HashMap<Id, ColorRole>,
    pub foreground: ForegroundColors,
    pub background: BackgroundColors,
    pub border: BorderColors,
    pub typefaces: HashMap<Id, Typeface>,
    pub text_styles: HashMap<Id, TextStyle>,
}

impl ThemeDefinition {
    pub fn load(stack: &PackStack) -> anyhow::Result<ThemeDefinition> {
        let mut palette = Palette::new(stack);
        let surface = |path: &str| Id {
            namespace: THEME_NAMESPACE.to_string(),
            path: path.to_string(),
        };

        let color_roles = stack
            .resource_ids(COLOR_ROLES)
            .into_iter()
            .map(|id| {
                let role = ThemeDefinition::read(stack, COLOR_ROLES, &id, |document, context| {
                    ColorRole::parse(document, context, &mut palette)
                })?;
                Ok((id, role))
            })
            .collect::<anyhow::Result<_>>()?;
        let foreground = ThemeDefinition::read(stack, SURFACES, &surface("foreground"), |document, context| {
            ForegroundColors::parse(document, context, &mut palette)
        })?;
        let background = ThemeDefinition::read(stack, SURFACES, &surface("background"), |document, context| {
            BackgroundColors::parse(document, context, &mut palette)
        })?;
        let border = ThemeDefinition::read(stack, SURFACES, &surface("border"), |document, context| {
            BorderColors::parse(document, context, &mut palette)
        })?;
        let typefaces: HashMap<Id, Typeface> = stack
            .resource_ids(TYPEFACES)
            .into_iter()
            .map(|id| {
                Ok((
                    id.clone(),
                    ThemeDefinition::read(stack, TYPEFACES, &id, Typeface::parse)?,
                ))
            })
            .collect::<anyhow::Result<_>>()?;
        let text_styles: HashMap<Id, TextStyle> = stack
            .resource_ids(TEXT_STYLES)
            .into_iter()
            .map(|id| {
                Ok((
                    id.clone(),
                    ThemeDefinition::read(stack, TEXT_STYLES, &id, TextStyle::parse)?,
                ))
            })
            .collect::<anyhow::Result<_>>()?;

        for (id, style) in &text_styles {
            ensure!(
                typefaces.contains_key(&style.typeface),
                "text style {id} uses typeface {}, which no loaded pack provides",
                style.typeface
            );
        }

        Ok(ThemeDefinition {
            color_roles,
            foreground,
            background,
            border,
            typefaces,
            text_styles,
        })
    }

    fn read<T>(
        stack: &PackStack,
        kind: &str,
        id: &Id,
        parse: impl FnOnce(&Document, &ParseContext) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        let path = stack.resource(kind, id, DOCUMENT_EXTENSION)?;
        let document = Document::read(path)?;
        let context = ParseContext { stack, id };

        parse(&document, &context).with_context(|| format!("{}", path.display()))
    }
}
