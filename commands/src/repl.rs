use std::io;
use std::thread;

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{Context, Editor, Helper, Highlighter, Hinter, Validator};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use usage::complete::{self, Shell};

use crate::command::CommandLine;
use crate::dispatcher::{Invocation, Output};
use crate::words;

const PROMPT: &str = "> ";
// Completion splits a full command line, whose first word is the program.
const PROGRAM_WORD: &str = "xsa ";

pub fn spawn(invocations: UnboundedSender<Invocation>) -> io::Result<()> {
    thread::Builder::new()
        .name("repl".into())
        .spawn(move || run(invocations))?;
    Ok(())
}

fn run(invocations: UnboundedSender<Invocation>) {
    let mut editor = match Editor::new() {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("the console is unavailable: {err}");
            return;
        }
    };
    editor.set_helper(Some(CommandCompleter));
    loop {
        let line = match editor.readline(PROMPT) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => return,
            Err(err) => {
                eprintln!("the console stopped: {err}");
                return;
            }
        };
        let words = match words::split(&line) {
            Ok(words) if words.is_empty() => continue,
            Ok(words) => words,
            Err(err) => {
                println!("{err}");
                continue;
            }
        };
        let _ = editor.add_history_entry(line);
        let (reply, receiver) = oneshot::channel();
        if invocations.send(Invocation { words, reply }).is_err() {
            return;
        }
        match receiver.blocking_recv() {
            Ok(output) => print(&output),
            Err(_) => println!("the command was dropped before it finished"),
        }
    }
}

fn print(output: &Output) {
    println!("{}", output.text.trim_end());
}

#[derive(Helper, Hinter, Highlighter, Validator)]
struct CommandCompleter;

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, position: usize, _context: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        let full_line = format!("{PROGRAM_WORD}{line}");
        let split = complete::split(&full_line, PROGRAM_WORD.len() + position, Shell::Bash);
        let candidates = complete::candidates(CommandLine::spec(), &split)
            .into_iter()
            .filter(|candidate| candidate.value.starts_with(&split.prefix))
            .map(|candidate| Pair {
                display: candidate.value.clone(),
                replacement: format!("{} ", candidate.value),
            })
            .collect();
        Ok((position - split.prefix.len(), candidates))
    }
}
