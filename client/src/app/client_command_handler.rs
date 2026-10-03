use super::App;

pub(super) trait ClientCommandHandler {
    fn run(self, app: &mut App) -> anyhow::Result<String>;
}
