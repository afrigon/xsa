use std::collections::HashMap;

use anyhow::Context;

use crate::{DOCUMENT_EXTENSION, Document, Id, PackStack};

use super::{ColorReference, Hue, SrgbColor};

const COLORS: &str = "theme/colors";

pub struct Palette<'a> {
    stack: &'a PackStack,
    hues: HashMap<Id, Hue>,
}

impl<'a> Palette<'a> {
    pub fn new(stack: &'a PackStack) -> Palette<'a> {
        Palette {
            stack,
            hues: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, reference: &ColorReference) -> anyhow::Result<SrgbColor> {
        if !self.hues.contains_key(&reference.hue) {
            let path = self
                .stack
                .resource(COLORS, &reference.hue, DOCUMENT_EXTENSION)
                .with_context(|| format!("unknown hue {}", reference.hue))?;
            let document = Document::read(path)?;
            let hue = Hue::parse(&document).with_context(|| format!("{}", path.display()))?;
            self.hues.insert(reference.hue.clone(), hue);
        }

        Ok(self.hues[&reference.hue].color(reference.tint))
    }
}
