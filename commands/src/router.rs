mod command_execution;
mod command_executor;
mod command_invocation;
mod output;

pub use command_execution::CommandExecution;
pub use command_executor::CommandExecutor;
pub use command_invocation::CommandInvocation;
pub use output::Output;

use std::collections::HashMap;
use std::ffi::OsStr;

use tokio::sync::oneshot;
use usage::help::Style;
use xsa_proto::event::{Outcome, ServerEvent};
use xsa_proto::message::MessageId;
use xsa_proto::session::ServerSession;

use crate::command::{CommandLine, Routable, Route, ServerCommand};
use crate::completion::Completions;

#[derive(Default)]
pub struct CommandRouter {
    pending: HashMap<MessageId, Pending>,
}

struct Pending {
    command: Box<dyn ServerCommand>,
    reply: oneshot::Sender<Output>,
}

impl CommandRouter {
    pub fn handle(&mut self, invocation: CommandInvocation, executor: &mut impl CommandExecutor) {
        match invocation {
            CommandInvocation::Execute(execution) => self.execute(execution, executor),
            CommandInvocation::Complete(completion) => {
                let completions = Completions::compute(&completion.line, completion.cursor, |kind| {
                    executor.completion_values(kind)
                });
                let _ = completion.reply.send(completions);
            }
        }
    }

    pub fn execute(&mut self, invocation: CommandExecution, executor: &mut impl CommandExecutor) {
        let words: Vec<&OsStr> = invocation.words.iter().map(OsStr::new).collect();
        let command = match CommandLine::parse_from(&words) {
            Ok(line) => line.command,
            Err(error) => {
                let output = CommandRouter::render_parse_error(&words, &error, invocation.styled);
                let _ = invocation.reply.send(output);

                return;
            }
        };
        let output = match command.route() {
            #[cfg(feature = "client")]
            Route::Client(command) => match executor.run_client(command) {
                Ok(text) => Output::success(text),
                Err(err) => Output::failure(format!("{err:#}")),
            },
            Route::Server(command) => match executor.session().send(command.message()) {
                Ok(id) => {
                    let pending = Pending {
                        command,
                        reply: invocation.reply,
                    };
                    self.pending.insert(id, pending);

                    return;
                }
                Err(err) => Output::failure(format!("{err:#}")),
            },
            Route::Session(command) => command.answer(executor.session().state()),
            Route::Exit => {
                executor.exit();
                Output::success("exiting")
            }
        };

        let _ = invocation.reply.send(output);
    }

    pub fn handle_event(&mut self, event: &ServerEvent, session: &ServerSession) {
        let ServerEvent::Reply(reply) = event else {
            return;
        };
        let Some(pending) = self.pending.remove(&reply.id) else {
            return;
        };
        let output = match &reply.outcome {
            Outcome::Accepted => pending.command.describe(session.state()),
            Outcome::Denied { reason } => Output::failure(reason.clone()),
        };

        let _ = pending.reply.send(output);
    }

    fn render_parse_error(words: &[&OsStr], error: &usage::Error<'static, '_>, styled: bool) -> Output {
        let style = if styled { Style::auto() } else { Style::PLAIN };

        match error {
            usage::Error::Help { cmd, long } => {
                Output::success(CommandLine::render_help_styled(cmd, *long, style).unwrap_or_default())
            }
            error if styled => Output::failure(CommandLine::render_failure(words, error)),
            error => Output::failure(usage::render_failure_plain(CommandLine::spec(), words, error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use xsa_proto::connection::{ClientLink, Connection};
    use xsa_proto::event::{JoinAccepted, Reply, TimeChanged, WorldState};
    use xsa_proto::message::{ClientFrame, ClientMessage, Join, Role, SetTime, SetTimeRate, StepTime};
    use xsa_units::{SimulationDuration, SimulationTime, TimeRate};

    use super::*;

    struct FakeExecutor {
        session: ServerSession,
        exited: bool,
    }

    impl CommandExecutor for FakeExecutor {
        fn session(&mut self) -> &mut ServerSession {
            &mut self.session
        }

        fn exit(&mut self) {
            self.exited = true;
        }
    }

    struct Harness {
        executor: FakeExecutor,
        link: ClientLink,
        router: CommandRouter,
    }

    impl Harness {
        fn joined() -> Self {
            let local = Connection::local();
            let mut harness = Self {
                executor: FakeExecutor {
                    session: ServerSession::new(local.connection),
                    exited: false,
                },
                link: local.link,
                router: CommandRouter::default(),
            };
            harness
                .executor
                .session
                .send(ClientMessage::Join(Join { role: Role::Server }))
                .unwrap();
            harness.link.messages.try_recv().unwrap();
            harness.deliver(ServerEvent::JoinAccepted(JoinAccepted {
                state: WorldState {
                    simulation: "system-solar:sol".to_string(),
                    packs: Vec::new(),
                    time: SimulationTime::J2000,
                    rate: TimeRate::PAUSED,
                    players: Vec::new(),
                },
            }));
            harness
        }

        fn deliver(&mut self, event: ServerEvent) {
            self.link.events.send(event).unwrap();
            while let Some(event) = self.executor.session.poll().unwrap() {
                self.router.handle_event(&event, &self.executor.session);
            }
        }

        fn invoke(&mut self, line: &str) -> oneshot::Receiver<Output> {
            let (reply, receiver) = oneshot::channel();
            let words = crate::words::split(line).unwrap();
            self.router.execute(
                CommandExecution {
                    words,
                    reply,
                    styled: false,
                },
                &mut self.executor,
            );
            receiver
        }

        fn sent(&mut self) -> ClientFrame {
            self.link.messages.try_recv().unwrap()
        }
    }

    #[test]
    fn time_shows_the_session_time() {
        let mut harness = Harness::joined();
        let output = harness.invoke("time").try_recv().unwrap();
        assert_eq!(output, Output::success("time: 2000-01-01T12:00:00.000Z, paused"));
    }

    #[test]
    fn server_commands_become_messages_answered_by_the_reply() {
        let mut harness = Harness::joined();
        let mut receiver = harness.invoke("time rate 100");
        let frame = harness.sent();
        let rate = TimeRate { multiplier: 100.0 };
        assert_eq!(frame.message, ClientMessage::SetTimeRate(SetTimeRate { rate }));
        assert!(receiver.try_recv().is_err());

        harness.deliver(ServerEvent::TimeChanged(TimeChanged {
            time: SimulationTime::J2000,
            rate,
        }));
        harness.deliver(ServerEvent::Reply(Reply {
            id: frame.id,
            outcome: Outcome::Accepted,
        }));
        let output = receiver.try_recv().unwrap();
        assert!(output.succeeded && output.text.contains("rate 100×"), "{output:?}");
    }

    #[test]
    fn a_denied_reply_fails_with_its_reason() {
        let mut harness = Harness::joined();
        let mut receiver = harness.invoke("time rate 5");
        let frame = harness.sent();
        harness.deliver(ServerEvent::Reply(Reply {
            id: frame.id,
            outcome: Outcome::denied("no"),
        }));
        assert_eq!(receiver.try_recv().unwrap(), Output::failure("no"));
    }

    struct Case {
        line: &'static str,
        message: ClientMessage,
    }

    #[test]
    fn aliases_and_durations_map_to_their_messages() {
        let mut harness = Harness::joined();
        let cases = [
            Case {
                line: "time pause",
                message: ClientMessage::SetTimeRate(SetTimeRate { rate: TimeRate::PAUSED }),
            },
            Case {
                line: "time resume",
                message: ClientMessage::SetTimeRate(SetTimeRate {
                    rate: TimeRate::REAL_TIME,
                }),
            },
            Case {
                line: "time step 2min",
                message: ClientMessage::StepTime(StepTime {
                    duration: SimulationDuration { seconds: 120.0 },
                }),
            },
            Case {
                line: "time set 2000-01-01T12:00:10Z",
                message: ClientMessage::SetTime(SetTime {
                    time: SimulationTime { seconds: 10.0 },
                }),
            },
        ];
        for case in cases {
            harness.invoke(case.line);
            assert_eq!(harness.sent().message, case.message, "{}", case.line);
        }
    }

    #[test]
    fn parse_errors_and_help_come_back_as_output() {
        let mut harness = Harness::joined();
        let error = harness.invoke("time rate fast").try_recv().unwrap();
        assert!(!error.succeeded && error.text.contains("fast"), "{error:?}");
        let help = harness.invoke("time --help").try_recv().unwrap();
        assert!(help.succeeded && help.text.contains("rate"), "{help:?}");
    }

    #[test]
    fn exit_asks_the_executor_to_exit() {
        let mut harness = Harness::joined();
        harness.invoke("exit");
        assert!(harness.executor.exited);
    }
}
