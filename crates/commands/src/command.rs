#[cfg(feature = "client")]
mod camera;
#[cfg(feature = "client")]
mod client_command;
mod command_line;
mod config;
mod exit;
#[cfg(feature = "client")]
mod input;
mod local_command;
mod routable;
mod route;
mod script;
mod server_command;
mod session_command;
mod time;
#[cfg(feature = "client")]
mod ui;
mod wait;

#[cfg(feature = "client")]
pub use camera::{
    CameraAction, CameraCaptureAction, CameraCaptureCommand, CameraCaptureStartCommand, CameraCaptureStopCommand,
    CameraCommand, CameraLookAtCommand, CameraTargetCommand, CameraTeleportCommand, DEFAULT_CAPTURE_FPS,
    TargetSelection,
};
#[cfg(feature = "client")]
pub use client_command::ClientCommand;
pub use command_line::CommandLine;
pub use config::{
    ConfigAction, ConfigCommand, ConfigGetCommand, ConfigLoadCommand, ConfigSaveCommand, ConfigSetCommand,
    ConfigToggleCommand, ConfigUnsetCommand,
};
pub use exit::ExitCommand;
#[cfg(feature = "client")]
pub use input::{
    InputAction, InputCommand, InputCursorAction, InputCursorCommand, InputCursorMoveCommand, InputCursorSetCommand,
    InputDownCommand, InputPressCommand, InputResetCommand, InputScrollCommand, InputUpCommand,
};
pub use local_command::LocalCommand;
pub use routable::Routable;
pub use route::Route;
pub use script::ScriptCommand;
pub use server_command::ServerCommand;
pub use session_command::SessionCommand;
pub use time::{
    TimeAction, TimeCommand, TimeFasterCommand, TimeOverrideCommand, TimePauseCommand, TimeRateCommand,
    TimeResumeCommand, TimeSetCommand, TimeSlowerCommand, TimeStepCommand,
};
#[cfg(feature = "client")]
pub use time::{TimeWarpCommand, TimeWarpRequestCommand};
#[cfg(feature = "client")]
pub use ui::{
    InterfaceAction, InterfaceCommand, InterfaceHideCommand, InterfacePopCommand, InterfacePushCommand,
    InterfaceSetCommand, InterfaceShowCommand, InterfaceToggleCommand, Overlay, ViewControllerId,
};
pub use wait::WaitCommand;

use usage::Subcommands;

#[derive(Subcommands)]
pub enum Command {
    /// Show or change the simulation time
    Time(TimeCommand),
    #[cfg(feature = "client")]
    /// Control the camera
    Camera(CameraCommand),
    /// Show, change, save or load this process's settings
    Config(ConfigCommand),
    #[cfg(feature = "client")]
    /// Change the interface's view controllers and overlays
    Ui(InterfaceCommand),
    #[cfg(feature = "client")]
    /// Inject keyboard and mouse input as if it came from the player's devices
    Input(InputCommand),
    /// Run commands one after another, each once the previous one replied
    Script(ScriptCommand),
    /// Wait before replying, to time the commands of a script
    Wait(WaitCommand),
    /// Exit
    Exit(ExitCommand),
}

impl Routable for Command {
    fn route(self) -> Route {
        match self {
            Command::Time(command) => command.route(),
            #[cfg(feature = "client")]
            Command::Camera(command) => command.route(),
            Command::Config(command) => command.route(),
            #[cfg(feature = "client")]
            Command::Ui(command) => command.route(),
            #[cfg(feature = "client")]
            Command::Input(command) => command.route(),
            Command::Script(command) => command.route(),
            Command::Wait(command) => command.route(),
            Command::Exit(command) => command.route(),
        }
    }
}
