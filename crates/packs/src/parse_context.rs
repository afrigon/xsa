use std::path::PathBuf;

use crate::{Id, PackStack};

pub struct ParseContext<'a> {
    pub stack: &'a PackStack,
    pub id: &'a Id,
}

impl ParseContext<'_> {
    pub fn parse_id(&self, text: &str) -> anyhow::Result<Id> {
        Id::parse(text, &self.id.namespace)
    }

    // A texture no loaded pack provides is skipped with a warning, so content loads while its textures are missing.
    pub fn available_texture(&self, text: &str) -> anyhow::Result<Option<PathBuf>> {
        let id = self.parse_id(text)?;

        match self.stack.texture(&id) {
            Ok(path) => Ok(Some(path.to_path_buf())),
            Err(err) => {
                tracing::warn!("{}: {err:#}", self.id);
                Ok(None)
            }
        }
    }
}
