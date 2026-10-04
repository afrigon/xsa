use std::collections::HashMap;
use std::fs;

use anyhow::Context;
use xsa_packs::{ForegroundColors, Id, PackStack, ThemeDefinition, ThemedColor};
use xui::{ColorScheme, Environment, Font, FontLibrary, LinearColor};

pub struct Theme {
    definition: ThemeDefinition,
    families: HashMap<Id, String>,
}

impl Theme {
    pub fn load(stack: &PackStack, fonts: &mut FontLibrary) -> anyhow::Result<Theme> {
        let definition = ThemeDefinition::load(stack)?;
        let mut families = HashMap::new();

        for (id, typeface) in &definition.typefaces {
            let mut family = None;

            for path in &typeface.fonts {
                let data = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
                let names = fonts.register(data).with_context(|| format!("{}", path.display()))?;
                family = family.or(names.into_iter().next());
            }

            families.insert(
                id.clone(),
                family.with_context(|| format!("typeface {id} names no font family"))?,
            );
        }

        Ok(Theme { definition, families })
    }

    pub fn font(&self, text_style: &Id) -> anyhow::Result<Font> {
        let style = self
            .definition
            .text_styles
            .get(text_style)
            .with_context(|| format!("no loaded pack provides text style {text_style}"))?;
        let family = self
            .families
            .get(&style.typeface)
            .with_context(|| format!("typeface {} was not loaded", style.typeface))?;

        Ok(Font::new(family.clone(), style.size)
            .weight(style.weight)
            .letter_spacing(style.letter_spacing)
            .features(style.features.clone()))
    }

    pub fn foreground(&self) -> &ForegroundColors {
        &self.definition.foreground
    }

    pub fn color(&self, color: &ThemedColor, environment: &Environment) -> LinearColor {
        let srgb = match environment.color_scheme {
            ColorScheme::Light => color.light,
            ColorScheme::Dark => color.dark,
        };

        LinearColor::from_srgb(srgb.red, srgb.green, srgb.blue)
    }
}
