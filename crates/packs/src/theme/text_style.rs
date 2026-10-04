use anyhow::{Context, ensure};

use crate::{Document, Id, NodeExtension, ParseContext};

use super::TextCase;

const FEATURE_TAG_LENGTH: usize = 4;

pub struct TextStyle {
    pub typeface: Id,
    pub size: f32,
    pub weight: f32,
    pub letter_spacing: f32,
    pub case: Option<TextCase>,
    pub features: Vec<String>,
}

impl TextStyle {
    pub fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<TextStyle> {
        let number = |name: &str| -> anyhow::Result<f32> { Ok(document.node(name)?.number_argument()? as f32) };
        let features = match document.optional_node("features") {
            None => Vec::new(),
            Some(node) => node
                .arguments()
                .map(|value| {
                    let tag = value
                        .as_string()
                        .context("`features` lists feature tags like \"tnum\"")?;
                    ensure!(
                        tag.len() == FEATURE_TAG_LENGTH && tag.is_ascii(),
                        "{tag:?} is not an OpenType feature tag like \"tnum\""
                    );
                    Ok(tag.to_string())
                })
                .collect::<anyhow::Result<_>>()?,
        };

        Ok(TextStyle {
            typeface: context.parse_id(document.node("typeface")?.string_argument()?)?,
            size: number("size")?,
            weight: number("weight")?,
            letter_spacing: match document.optional_node("letter-spacing") {
                Some(node) => node.number_argument()? as f32,
                None => 0.0,
            },
            case: document
                .optional_node("case")
                .map(|node| TextCase::parse(node.string_argument()?))
                .transpose()?,
            features,
        })
    }
}
