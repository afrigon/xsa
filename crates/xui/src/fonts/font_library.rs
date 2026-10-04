use std::sync::Arc;

use anyhow::ensure;
use parley::FontContext;
use parley::fontique::{Blob, Collection, CollectionOptions, SourceCache};

pub struct FontLibrary {
    context: FontContext,
}

impl FontLibrary {
    pub fn new() -> FontLibrary {
        FontLibrary {
            context: FontContext {
                collection: Collection::new(CollectionOptions {
                    system_fonts: false,
                    ..CollectionOptions::default()
                }),
                source_cache: SourceCache::default(),
            },
        }
    }

    pub fn register(&mut self, data: Vec<u8>) -> anyhow::Result<Vec<String>> {
        let families = self.context.collection.register_fonts(Blob::new(Arc::new(data)), None);
        ensure!(!families.is_empty(), "the data holds no font");

        let mut names = Vec::new();

        for (family, _) in families {
            if let Some(name) = self.context.collection.family_name(family) {
                names.push(name.to_string());
            }
        }

        Ok(names)
    }

    pub(crate) fn context_mut(&mut self) -> &mut FontContext {
        &mut self.context
    }
}

impl Default for FontLibrary {
    fn default() -> FontLibrary {
        FontLibrary::new()
    }
}
