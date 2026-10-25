use std::fs;
use std::path::Path;

use anyhow::{Context, anyhow};

use crate::words;

const TEXT_NAME: &str = "script";
const COMMAND_SEPARATOR: char = ';';

pub struct Script {
    name: String,
    lines: Vec<ScriptLine>,
}

pub struct ScriptLine {
    pub number: usize,
    pub words: Vec<String>,
}

impl Script {
    pub fn load(path: &Path) -> anyhow::Result<Script> {
        let text = fs::read_to_string(path).with_context(|| format!("failed to read the script {}", path.display()))?;
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        );

        Script::parse(name, &text)
    }

    // Commands are separated by newlines or semicolons; each counts as a line.
    pub fn from_text(text: &str) -> anyhow::Result<Script> {
        Script::parse(TEXT_NAME.to_string(), &Script::separate_commands(text))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn line(&self, index: usize) -> Option<&ScriptLine> {
        self.lines.get(index)
    }

    fn parse(name: String, text: &str) -> anyhow::Result<Script> {
        let mut lines = Vec::new();

        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            let words = words::split(line).map_err(|err| anyhow!("{name}:{number}: {err}"))?;

            if !words.is_empty() {
                lines.push(ScriptLine { number, words });
            }
        }

        Ok(Script { name, lines })
    }

    // A quoted or escaped semicolon belongs to its word.
    fn separate_commands(text: &str) -> String {
        let mut separated = String::with_capacity(text.len());
        let mut quote = None;
        let mut escaped = false;

        for character in text.chars() {
            match character {
                _ if escaped => escaped = false,
                '\\' if quote != Some('\'') => escaped = true,
                '\'' | '"' if quote == Some(character) => quote = None,
                '\'' | '"' if quote.is_none() => quote = Some(character),
                COMMAND_SEPARATOR if quote.is_none() => {
                    separated.push('\n');
                    continue;
                }
                _ => {}
            }

            separated.push(character);
        }

        separated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Script {
        Script::parse("init.xsa".to_string(), text).unwrap()
    }

    fn words(script: &Script) -> Vec<(usize, Vec<&str>)> {
        script
            .lines
            .iter()
            .map(|line| (line.number, line.words.iter().map(String::as_str).collect()))
            .collect()
    }

    #[test]
    fn blank_lines_and_comments_are_skipped_and_line_numbers_kept() {
        let script = parse("ui push game\n\n# stand on the Moon\ncamera tp --lat 0 luna # at the equator\n");

        assert_eq!(
            words(&script),
            vec![
                (1, vec!["ui", "push", "game"]),
                (4, vec!["camera", "tp", "--lat", "0", "luna"])
            ]
        );
    }

    #[test]
    fn an_unclosed_quote_names_its_line() {
        let error = Script::parse("init.xsa".to_string(), "ui push game\nconfig set \"x\n")
            .err()
            .unwrap();

        assert!(error.to_string().starts_with("init.xsa:2: "), "{error}");
    }

    #[test]
    fn semicolons_separate_text_commands_unless_quoted() {
        let script = Script::from_text("input down w; wait 50ms;input up w; config set a 'b;c' \"d;e\" f\\;g").unwrap();

        assert_eq!(script.name(), "script");
        assert_eq!(
            words(&script),
            vec![
                (1, vec!["input", "down", "w"]),
                (2, vec!["wait", "50ms"]),
                (3, vec!["input", "up", "w"]),
                (4, vec!["config", "set", "a", "b;c", "d;e", "f;g"]),
            ]
        );
    }
}
