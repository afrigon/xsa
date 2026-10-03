#[cfg(feature = "client")]
mod camera;
#[cfg(feature = "client")]
mod client_command;
mod command_line;
mod exit;
mod routable;
mod route;
mod server_command;
mod session_command;
mod time;

#[cfg(feature = "client")]
pub use camera::{
    CameraAction, CameraCommand, CameraLookAtCommand, CameraMode, CameraModeCommand, CameraTargetCommand,
};
#[cfg(feature = "client")]
pub use client_command::ClientCommand;
pub use command_line::CommandLine;
pub use exit::ExitCommand;
pub use routable::Routable;
pub use route::Route;
pub use server_command::ServerCommand;
pub use session_command::SessionCommand;
pub use time::{
    TimeAction, TimeCommand, TimePauseCommand, TimeRateCommand, TimeResumeCommand, TimeSetCommand, TimeStepCommand,
};

use usage::Subcommands;

#[derive(Subcommands)]
pub enum Command {
    /// Show or change the simulation time
    Time(TimeCommand),
    #[cfg(feature = "client")]
    /// Control the camera
    Camera(CameraCommand),
    /// Exit
    Exit(ExitCommand),
}

impl Routable for Command {
    fn route(self) -> Route {
        match self {
            Command::Time(command) => command.route(),
            #[cfg(feature = "client")]
            Command::Camera(command) => command.route(),
            Command::Exit(command) => command.route(),
        }
    }
}
