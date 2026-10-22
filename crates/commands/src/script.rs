use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{Context, anyhow};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::router::{CommandExecution, CommandInvocation};
use crate::words;

pub struct Script {
    path: PathBuf,
    lines: Vec<ScriptLine>,
}

struct ScriptLine {
    number: usize,
    words: Vec<String>,
}

impl Script {
    pub fn load(path: &Path) -> anyhow::Result<Script> {
        let text = fs::read_to_string(path).with_context(|| format!("failed to read the script {}", path.display()))?;

        Script::parse(path.to_path_buf(), &text)
    }

    fn parse(path: PathBuf, text: &str) -> anyhow::Result<Script> {
        let mut lines = Vec::new();

        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            let words = words::split(line).map_err(|err| anyhow!("{}:{number}: {err}", path.display()))?;

            if !words.is_empty() {
                lines.push(ScriptLine { number, words });
            }
        }

        Ok(Script { path, lines })
    }

    pub fn spawn(self, invocations: UnboundedSender<CommandInvocation>) -> io::Result<()> {
        thread::Builder::new()
            .name("script".into())
            .spawn(move || self.run(invocations))?;

        Ok(())
    }

    fn run(self, invocations: UnboundedSender<CommandInvocation>) {
        for line in self.lines {
            let location = format!("{}:{}", self.path.display(), line.number);
            let (reply, receiver) = oneshot::channel();
            let invocation = CommandInvocation::Execute(CommandExecution {
                words: line.words,
                reply,
                styled: false,
            });

            if invocations.send(invocation).is_err() {
                return;
            }

            match receiver.blocking_recv() {
                Ok(output) if output.succeeded => tracing::info!("{location}: {}", output.text.trim_end()),
                Ok(output) => {
                    tracing::error!(
                        "{location}: {}; skipping the rest of the script",
                        output.text.trim_end()
                    );
                    return;
                }
                Err(_) => {
                    tracing::warn!("{location}: the command was dropped before it finished");
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::unbounded_channel;

    use super::*;
    use crate::router::Output;

    fn parse(text: &str) -> Script {
        Script::parse(PathBuf::from("init.xsa"), text).unwrap()
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
        let error = Script::parse(PathBuf::from("init.xsa"), "ui push game\nconfig set \"x\n")
            .err()
            .unwrap();

        assert!(error.to_string().starts_with("init.xsa:2: "), "{error}");
    }

    #[test]
    fn each_line_waits_for_the_previous_reply_and_a_failure_stops_the_script() {
        let script = parse("first\nsecond\nthird\n");
        let (sender, mut invocations) = unbounded_channel();
        let runner = thread::spawn(move || script.run(sender));
        let mut received = Vec::new();

        while let Some(CommandInvocation::Execute(execution)) = invocations.blocking_recv() {
            assert!(
                invocations.is_empty(),
                "a line was sent before the previous one replied"
            );

            let output = match execution.words[0].as_str() {
                "second" => Output::failure("no"),
                _ => Output::success("ok"),
            };
            received.push(execution.words[0].clone());
            execution.reply.send(output).unwrap();
        }

        runner.join().unwrap();

        assert_eq!(received, vec!["first", "second"]);
    }
}
