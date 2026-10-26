use anyhow::anyhow;
use xsa_commands::command::CommandLine;
use xsa_commands::words;
use xsa_config::{ConfigValue, KdlValue};

// A command a bind runs, checked when it is set or loaded: a bind never holds a command that does not parse.
#[derive(Clone, Debug, PartialEq)]
pub struct BindCommand {
    text: String,
    words: Vec<String>,
}

impl BindCommand {
    pub fn words(&self) -> &[String] {
        &self.words
    }
}

impl ConfigValue for BindCommand {
    fn from_kdl(value: &KdlValue) -> Option<BindCommand> {
        value.as_string().and_then(|text| BindCommand::parse(text).ok())
    }

    fn to_kdl(&self) -> KdlValue {
        KdlValue::String(self.text.clone())
    }

    fn parse(text: &str) -> anyhow::Result<BindCommand> {
        let words = words::split(text).map_err(|err| anyhow!("{err}"))?;
        CommandLine::parse_words(&words).map_err(|reason| anyhow!("{text:?} is not a command: {reason}"))?;

        Ok(BindCommand {
            text: text.to_string(),
            words,
        })
    }

    fn type_name() -> String {
        "command".to_string()
    }
}
