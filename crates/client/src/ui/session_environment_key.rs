use xsa_proto::session::SessionEnvironment;
use xui::EnvironmentKey;

pub struct SessionEnvironmentKey;

impl EnvironmentKey for SessionEnvironmentKey {
    type Value = Option<SessionEnvironment>;

    fn default_value() -> Option<SessionEnvironment> {
        None
    }
}
