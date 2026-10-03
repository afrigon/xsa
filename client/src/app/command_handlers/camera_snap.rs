use xsa_commands::command::CameraSnapCommand;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::snapshot::Snapshot;

impl ClientCommandHandler for CameraSnapCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let mut snapshot = Snapshot::new(app.capture()?)?;

        if let Some(region) = self.region {
            snapshot = snapshot.crop(region)?;
        }

        let path = match self.output {
            Some(path) => path,
            None => Snapshot::default_path()?,
        };

        snapshot.save(&path)
    }
}
