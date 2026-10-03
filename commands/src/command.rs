use usage::{Args, Cli, Subcommands};
use xsa_proto::messages::ClientMessage;

use crate::duration::Duration;
use crate::timestamp::Timestamp;
#[cfg(feature = "client")]
use crate::{completion::complete_target, distance::Distance, target::Target};

#[derive(Cli)]
#[usage(bin = "", unknown_flags = "error", args_override_self = false)]
pub struct CommandLine {
    #[usage(subcommand)]
    pub command: Command,
}

#[derive(Subcommands)]
pub enum Command {
    /// Show or change the simulation time
    Time(TimeCommand),
    #[cfg(feature = "client")]
    /// Control the camera
    Camera(CameraCommand),
    /// Exit
    Exit,
}

#[derive(Args)]
pub struct TimeCommand {
    #[usage(subcommand)]
    pub action: Option<TimeAction>,
}

#[derive(Subcommands)]
pub enum TimeAction {
    /// Jump to a UTC date and time, e.g. 2026-10-02T12:00:00Z
    Set { time: Timestamp },
    /// Run the simulation at a multiple of real time; 0 pauses
    Rate {
        #[usage(allow_negative_numbers)]
        multiplier: f64,
    },
    /// Stop time (time rate 0)
    Pause,
    /// Run time at real speed (time rate 1)
    Resume,
    /// Advance time by a duration: s, min, h, or t for ticks
    Step { duration: Duration },
}

#[cfg(feature = "client")]
#[derive(Args)]
pub struct CameraCommand {
    #[usage(subcommand)]
    pub action: CameraAction,
}

#[cfg(feature = "client")]
#[derive(Subcommands)]
pub enum CameraAction {
    /// Switch between the target camera and the debug fly camera
    Mode {
        #[usage(value_enum)]
        mode: CameraMode,
    },
    /// Orbit a body: a body id, next or previous
    Target(CameraTarget),
    /// Turn the debug camera towards a body
    LookAt {
        #[usage(complete = complete_target)]
        target: Target,
    },
}

#[cfg(feature = "client")]
#[derive(usage::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Target,
    Debug,
}

#[cfg(feature = "client")]
#[derive(Args)]
pub struct CameraTarget {
    #[usage(complete = complete_target)]
    pub target: Target,
    #[usage(
        long,
        help = "Distance from the body's center, e.g. 200km (m, km, Mm, Gm; a bare number is km)"
    )]
    pub distance: Option<Distance>,
    #[usage(long, allow_negative_numbers, help = "Pitch in degrees; positive looks up")]
    pub pitch: Option<f64>,
    #[usage(long, allow_negative_numbers, help = "Yaw in degrees around the ecliptic north pole")]
    pub yaw: Option<f64>,
}

#[cfg(feature = "client")]
pub enum ClientCommand {
    Camera(CameraAction),
}

pub enum Route {
    #[cfg(feature = "client")]
    Client(ClientCommand),
    Server(ClientMessage),
    ShowTime,
    Exit,
}

impl Command {
    pub fn route(self) -> Route {
        match self {
            Command::Time(TimeCommand { action: None }) => Route::ShowTime,
            Command::Time(TimeCommand { action: Some(action) }) => Route::Server(match action {
                TimeAction::Set { time } => ClientMessage::SetTime { time: time.time },
                TimeAction::Rate { multiplier } => ClientMessage::SetTimeRate { rate: multiplier },
                TimeAction::Pause => ClientMessage::SetTimeRate { rate: 0.0 },
                TimeAction::Resume => ClientMessage::SetTimeRate { rate: 1.0 },
                TimeAction::Step { duration } => ClientMessage::StepTime {
                    seconds: duration.seconds(),
                },
            }),
            #[cfg(feature = "client")]
            Command::Camera(CameraCommand { action }) => Route::Client(ClientCommand::Camera(action)),
            Command::Exit => Route::Exit,
        }
    }
}
