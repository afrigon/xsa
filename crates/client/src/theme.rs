mod theme_key;

pub use theme_key::ThemeKey;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;

use anyhow::Context;
use xsa_packs::{ColorRole, ForegroundColors, Id, PackStack, ThemeDefinition, ThemedColor};
use xui::{ColorScheme, Font, FontLibrary, LinearColor};

pub struct Theme {
    definition: ThemeDefinition,
    families: HashMap<Id, String>,
    reported_styles: RefCell<HashSet<Id>>,
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

        Ok(Theme {
            definition,
            families,
            reported_styles: RefCell::new(HashSet::new()),
        })
    }

    // An unknown style is reported once and leaves the caller's font unchanged.
    pub fn font(&self, text_style: &Id) -> Option<Font> {
        let style = self.definition.text_styles.get(text_style);
        let family = style.and_then(|style| self.families.get(&style.typeface));
        let (Some(style), Some(family)) = (style, family) else {
            if self.reported_styles.borrow_mut().insert(text_style.clone()) {
                tracing::warn!("no loaded pack provides text style {text_style}");
            }

            return None;
        };

        Some(
            Font::new(family.clone(), style.size)
                .weight(style.weight)
                .letter_spacing(style.letter_spacing)
                .features(style.features.clone()),
        )
    }

    pub fn foreground(&self) -> &ForegroundColors {
        &self.definition.foreground
    }

    pub fn color_role(&self, role: &Id) -> Option<&ColorRole> {
        self.definition.color_roles.get(role)
    }

    pub fn color(&self, color: &ThemedColor, scheme: ColorScheme) -> LinearColor {
        let srgb = match scheme {
            ColorScheme::Light => color.light,
            ColorScheme::Dark => color.dark,
        };

        LinearColor::from_srgb(srgb.red, srgb.green, srgb.blue)
    }
}
