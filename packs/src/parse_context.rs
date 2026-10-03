use crate::{Id, PackStack};

pub struct ParseContext<'a> {
    pub stack: &'a PackStack,
    pub id: &'a Id,
}

impl ParseContext<'_> {
    pub fn parse_id(&self, text: &str) -> anyhow::Result<Id> {
        Id::parse(text, &self.id.namespace)
    }
}
