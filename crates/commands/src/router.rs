mod command_execution;
mod command_executor;
mod command_invocation;
#[cfg(feature = "client")]
mod command_reply;
mod output;

pub use command_execution::CommandExecution;
pub use command_executor::CommandExecutor;
pub use command_invocation::CommandInvocation;
#[cfg(feature = "client")]
pub use command_reply::CommandReply;
pub use output::Output;

use std::collections::HashMap;
use std::ffi::OsStr;

use tokio::sync::oneshot;
use usage::help::Style;
use xsa_proto::event::{Event, Outcome};
use xsa_proto::request::RequestId;
use xsa_proto::session::ServerSession;

use crate::command::{Command, CommandLine, Routable, Route, ServerCommand};
use crate::completion::Completions;

#[derive(Default)]
pub struct CommandRouter {
    pending: HashMap<RequestId, Pending>,
    waiting_for_join: Vec<Waiting>,
}

struct Pending {
    command: Box<dyn ServerCommand>,
    reply: oneshot::Sender<Output>,
}

struct Waiting {
    route: Route,
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
        self.dispatch(command, invocation.reply, executor);
    }

    // Runs an already typed command, as the interface does, the same way as a command typed in.
    pub fn dispatch(&mut self, command: Command, reply: oneshot::Sender<Output>, executor: &mut impl CommandExecutor) {
        let route = command.route();
        let joined = executor.session().state().is_some();

        if !joined && !matches!(route, Route::Exit | Route::Local(_)) {
            self.waiting_for_join.push(Waiting { route, reply });

            return;
        }

        self.run(route, reply, executor);
    }

    pub fn resume(&mut self, executor: &mut impl CommandExecutor) {
        if executor.session().state().is_none() {
            return;
        }

        for waiting in std::mem::take(&mut self.waiting_for_join) {
            self.run(waiting.route, waiting.reply, executor);
        }
    }

    fn run(&mut self, route: Route, reply: oneshot::Sender<Output>, executor: &mut impl CommandExecutor) {
        let output = match route {
            #[cfg(feature = "client")]
            Route::Client(command) => {
                executor.run_client(command, CommandReply::new(reply));

                return;
            }
            Route::Local(command) => executor.run_local(command),
            Route::Server(command) => {
                let request = command.request();
                let refusal = executor
                    .session()
                    .state()
                    .and_then(|state| request.check_session_environment(state.environment()).err());

                if let Some(reason) = refusal {
                    Output::failure(reason)
                } else {
                    match executor.session().send(request) {
                        Ok(id) => {
                            let pending = Pending { command, reply };
                            self.pending.insert(id, pending);

                            return;
                        }
                        Err(err) => Output::failure(format!("{err:#}")),
                    }
                }
            }
            Route::Session(command) => command.answer(executor.session().state()),
            Route::Exit => {
                executor.exit();
                Output::success("exiting")
            }
        };

        let _ = reply.send(output);
    }

    pub fn handle_event(&mut self, event: &Event, session: &ServerSession) {
        let Event::Reply(reply) = event else {
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
    use xsa_proto::event::{JoinAccepted, Reply, TimeChangedEvent};
    use xsa_proto::frame::ClientFrame;
    use xsa_proto::network::{ClientLink, Connection};
    use xsa_proto::request::{JoinRequest, Request, Role, SetTimeRateRequest, SetTimeRequest, StepTimeRequest};
    use xsa_proto::session::SessionEnvironment;
    use xsa_proto::world::WorldState;
    use xsa_units::{SimulationDuration, SimulationTime, TimeFlow, TimeRate, TimeRateChange};

    use super::*;
    use crate::command::LocalCommand;

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

        fn run_local(&mut self, _command: LocalCommand) -> Output {
            Output::failure("no config in this test")
        }
    }

    struct Harness {
        executor: FakeExecutor,
        link: ClientLink,
        router: CommandRouter,
    }

    impl Harness {
        fn unjoined() -> Self {
            let local = Connection::local();
            Self {
                executor: FakeExecutor {
                    session: ServerSession::new(local.connection),
                    exited: false,
                },
                link: local.link,
                router: CommandRouter::default(),
            }
        }

        fn joined() -> Self {
            Harness::joined_in(SessionEnvironment::Singleplayer)
        }

        fn joined_in(environment: SessionEnvironment) -> Self {
            let mut harness = Harness::unjoined();
            harness
                .executor
                .session
                .send(Request::Join(JoinRequest { role: Role::Server }))
                .unwrap();
            harness.link.requests.try_recv().unwrap();
            harness.deliver(Event::JoinAccepted(JoinAccepted {
                state: WorldState {
                    simulation: "system-solar:sol".to_string(),
                    packs: Vec::new(),
                    time: SimulationTime::J2000,
                    time_flow: TimeFlow {
                        rate: TimeRate::PAUSED,
                        time_override: None,
                    },
                    environment,
                    users: Vec::new(),
                },
            }));
            harness
        }

        fn deliver(&mut self, event: Event) {
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
            self.link.requests.try_recv().unwrap()
        }
    }

    #[test]
    fn time_shows_the_session_time() {
        let mut harness = Harness::joined();
        let output = harness.invoke("time").try_recv().unwrap();
        assert_eq!(output, Output::success("time: 2000-01-01T12:00:00.000Z, paused"));
    }

    #[test]
    fn server_commands_become_requests_answered_by_the_reply() {
        let mut harness = Harness::joined();
        let mut receiver = harness.invoke("time rate 100");
        let frame = harness.sent();
        let rate = TimeRate { multiplier: 100.0 };
        assert_eq!(
            frame.request,
            Request::SetTimeRate(SetTimeRateRequest {
                change: TimeRateChange::To(rate)
            })
        );
        assert!(receiver.try_recv().is_err());

        harness.deliver(Event::TimeChanged(TimeChangedEvent {
            time: SimulationTime::J2000,
            time_flow: TimeFlow {
                rate,
                time_override: None,
            },
        }));
        harness.deliver(Event::Reply(Reply {
            id: frame.id,
            outcome: Outcome::Accepted,
        }));
        let output = receiver.try_recv().unwrap();
        assert!(output.succeeded && output.text.contains("rate 100×"), "{output:?}");
    }

    #[test]
    fn a_request_outside_its_session_environment_is_refused_without_sending() {
        let mut harness = Harness::joined_in(SessionEnvironment::Dedicated);
        let output = harness.invoke("time override 0").try_recv().unwrap();
        assert!(
            !output.succeeded && output.text.contains("dedicated server"),
            "{output:?}"
        );
        assert!(harness.link.requests.try_recv().is_err());
    }

    #[test]
    fn a_denied_reply_fails_with_its_reason() {
        let mut harness = Harness::joined();
        let mut receiver = harness.invoke("time rate 5");
        let frame = harness.sent();
        harness.deliver(Event::Reply(Reply {
            id: frame.id,
            outcome: Outcome::denied("no"),
        }));
        assert_eq!(receiver.try_recv().unwrap(), Output::failure("no"));
    }

    struct Case {
        line: &'static str,
        request: Request,
    }

    #[test]
    fn aliases_and_durations_map_to_their_requests() {
        let mut harness = Harness::joined();
        let cases = [
            Case {
                line: "time pause",
                request: Request::SetTimeRate(SetTimeRateRequest {
                    change: TimeRateChange::To(TimeRate::PAUSED),
                }),
            },
            Case {
                line: "time resume",
                request: Request::SetTimeRate(SetTimeRateRequest {
                    change: TimeRateChange::To(TimeRate::REAL_TIME),
                }),
            },
            Case {
                line: "time faster",
                request: Request::SetTimeRate(SetTimeRateRequest {
                    change: TimeRateChange::Faster,
                }),
            },
            Case {
                line: "time slower",
                request: Request::SetTimeRate(SetTimeRateRequest {
                    change: TimeRateChange::Slower,
                }),
            },
            Case {
                line: "time step 2min",
                request: Request::StepTime(StepTimeRequest {
                    duration: SimulationDuration { seconds: 120.0 },
                }),
            },
            Case {
                line: "time set 2000-01-01T12:00:10Z",
                request: Request::SetTime(SetTimeRequest {
                    time: SimulationTime { seconds: 10.0 },
                }),
            },
        ];

        for case in cases {
            harness.invoke(case.line);
            assert_eq!(harness.sent().request, case.request, "{}", case.line);
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
    fn commands_wait_for_the_join_but_exit_does_not() {
        let mut harness = Harness::unjoined();
        let mut time = harness.invoke("time");
        assert!(time.try_recv().is_err());

        harness.invoke("exit");
        assert!(harness.executor.exited);

        harness.deliver(Event::JoinAccepted(JoinAccepted {
            state: WorldState {
                simulation: "system-solar:sol".to_string(),
                packs: Vec::new(),
                time: SimulationTime::J2000,
                time_flow: TimeFlow {
                    rate: TimeRate::PAUSED,
                    time_override: None,
                },
                environment: SessionEnvironment::Singleplayer,
                users: Vec::new(),
            },
        }));
        harness.router.resume(&mut harness.executor);
        assert!(time.try_recv().unwrap().succeeded);
    }

    #[test]
    fn exit_asks_the_executor_to_exit() {
        let mut harness = Harness::joined();
        harness.invoke("exit");
        assert!(harness.executor.exited);
    }
}
