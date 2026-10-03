use crate::{Document, ParseContext};

pub trait PackData: Sized {
    const KIND: &'static str;

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<Self>;
}
