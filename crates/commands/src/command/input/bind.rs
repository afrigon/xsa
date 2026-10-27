use usage::Args;
use xsa_config::{ConfigChange, ConfigSection, ConfigStore};

use crate::command::{LocalCommand, Routable, Route};

const BIND_SECTION: &str = "input.bind";

#[derive(Args)]
pub struct InputBindCommand {
    #[usage(help = "The bind, by category and name: camera.zoom")]
    pub bind: String,
    #[usage(help = "The key chord that fires it: c, shift+tab")]
    pub key: String,
    #[usage(long, help = "The command it runs as the key goes down")]
    pub down: Option<String>,
    #[usage(long, help = "The command it runs as the key is released, or none")]
    pub up: Option<String>,
    #[usage(long, help = "Write the change to the config file in effect")]
    pub save: bool,
}

impl InputBindCommand {
    // The same as setting the bind's `key`, `down` and `up` one by one.
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &mut ConfigStore<T>) -> anyhow::Result<String> {
        let path = |field: &str| format!("{BIND_SECTION}.{}.{field}", self.bind);
        let (key, down, up) = (path("key"), path("down"), path("up"));
        let mut changes = vec![ConfigChange {
            path: &key,
            text: &self.key,
        }];

        if let Some(text) = &self.down {
            changes.push(ConfigChange { path: &down, text });
        }

        if let Some(text) = &self.up {
            changes.push(ConfigChange { path: &up, text });
        }

        store.set_all(&changes, self.save)?;

        let mut description = format!(
            "{} {} down={}",
            self.bind,
            store.get(&path("key"))?,
            store.get(&path("down"))?
        );
        description.push_str(&format!(" up={}", store.get(&path("up"))?));

        if self.save {
            description.push_str(&format!(" (saved to {})", store.path().display()));
        }

        Ok(description)
    }
}

impl Routable for InputBindCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::InputBind(self))
    }
}
