use anyhow::bail;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextCase {
    Upper,
    Lower,
}

impl TextCase {
    pub fn parse(text: &str) -> anyhow::Result<TextCase> {
        match text {
            "upper" => Ok(TextCase::Upper),
            "lower" => Ok(TextCase::Lower),
            other => bail!("unknown case `{other}`: use `upper` or `lower`"),
        }
    }
}
