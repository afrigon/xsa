mod command_completer;
mod interrupt_handler;

use std::io::{self, IsTerminal};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use rustyline::error::ReadlineError;
use rustyline::{Editor, EventHandler, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::console_log::ConsoleLog;
use crate::router::{CommandExecution, CommandInvocation};
use crate::terminal::TerminalGuard;
use crate::words;
use command_completer::CommandCompleter;
use interrupt_handler::InterruptHandler;

const PROMPT: &str = "> ";
const EXIT: &str = "exit";

pub struct Repl {
    _terminal: Option<TerminalGuard>,
}

impl Repl {
    pub fn spawn(invocations: UnboundedSender<CommandInvocation>) -> io::Result<Repl> {
        let terminal = TerminalGuard::capture();
        thread::Builder::new()
            .name("repl".into())
            .spawn(move || Repl::run(invocations))?;

        Ok(Repl { _terminal: terminal })
    }

    fn run(invocations: UnboundedSender<CommandInvocation>) {
        let mut editor = match Editor::new() {
            Ok(editor) => editor,
            Err(err) => {
                tracing::error!("the console is unavailable: {err}");
                return;
            }
        };

        let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();

        if interactive && let Ok(printer) = editor.create_external_printer() {
            ConsoleLog::attach(printer);
        }

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
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    tracing::error!("the console stopped: {err}");
                    break;
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
            let invocation = CommandInvocation::Execute(CommandExecution {
                words,
                reply,
                styled: true,
            });

            if invocations.send(invocation).is_err() {
                break;
            }

            match receiver.blocking_recv() {
                Ok(output) => println!("{}", output.text.trim_end()),
                Err(_) => println!("the command was dropped before it finished"),
            }
        }

        ConsoleLog::detach();
    }
}
