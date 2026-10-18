use crate::launchpad_context::LaunchpadContext;

pub trait CommandHandler {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()>;
}
