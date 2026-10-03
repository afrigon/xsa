use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{
    Cmd, ConditionalEventHandler, Context, Editor, Event, EventContext, EventHandler, Helper, Highlighter, Hinter,
    KeyEvent, RepeatCount, Validator,
};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use xsa_core::packs::id::NAMESPACE_SEPARATOR;

use crate::completion::CommandCompletion;
use crate::router::{CommandExecution, CommandInvocation, Output};
use crate::terminal::TerminalGuard;
use crate::words;

const PROMPT: &str = "> ";
const EXIT: &str = "exit";

pub struct Repl {
    _terminal: Option<TerminalGuard>,
}

pub fn spawn(invocations: UnboundedSender<CommandInvocation>) -> io::Result<Repl> {
    let terminal = TerminalGuard::capture();
    thread::Builder::new()
        .name("repl".into())
        .spawn(move || run(invocations))?;
    Ok(Repl { _terminal: terminal })
}

fn run(invocations: UnboundedSender<CommandInvocation>) {
    let mut editor = match Editor::new() {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("the console is unavailable: {err}");
            return;
        }
    };
    editor.set_helper(Some(CommandCompleter {
        invocations: invocations.clone(),
    }));
    let interrupted_empty_line = Arc::new(AtomicBool::new(false));
    editor.bind_sequence(
        KeyEvent::ctrl('C'),
        EventHandler::Conditional(Box::new(InterruptHandler {
            empty_line: interrupted_empty_line.clone(),
        })),
    );
    loop {
        let line = match editor.readline(PROMPT) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) if interrupted_empty_line.load(Ordering::Relaxed) => EXIT.to_string(),
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
        if invocations
            .send(CommandInvocation::Execute(CommandExecution {
                words,
                reply,
                styled: true,
            }))
            .is_err()
        {
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

// Ctrl-C clears a line being typed, like readline; on an empty line it exits, like the signal it replaces.
struct InterruptHandler {
    empty_line: Arc<AtomicBool>,
}

impl ConditionalEventHandler for InterruptHandler {
    fn handle(&self, _event: &Event, _count: RepeatCount, _positive: bool, context: &EventContext) -> Option<Cmd> {
        self.empty_line.store(context.line().is_empty(), Ordering::Relaxed);
        Some(Cmd::Interrupt)
    }
}

#[derive(Helper, Hinter, Highlighter, Validator)]
struct CommandCompleter {
    invocations: UnboundedSender<CommandInvocation>,
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, position: usize, _context: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (reply, receiver) = oneshot::channel();
        let completion = CommandCompletion {
            line: line.to_string(),
            cursor: position,
            reply,
        };
        if self.invocations.send(CommandInvocation::Complete(completion)).is_err() {
            return Ok((position, Vec::new()));
        }
        let Ok(completions) = receiver.blocking_recv() else {
            return Ok((position, Vec::new()));
        };
        let candidates = completions
            .candidates
            .into_iter()
            .map(|candidate| Pair {
                display: candidate.value.clone(),
                replacement: if candidate.value.ends_with(NAMESPACE_SEPARATOR) {
                    candidate.value
                } else {
                    format!("{} ", candidate.value)
                },
            })
            .collect();
        Ok((completions.start, candidates))
    }
}
