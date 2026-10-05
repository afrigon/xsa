use std::collections::HashMap;

use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::{BindAction, BindConfig, KeyChord};

pub struct BindDocument {
    keys: HashMap<BindAction, KeyChord>,
}

impl BindDocument {
    pub fn keys() -> Vec<ConfigKey> {
        BindAction::ALL.into_iter().map(BindDocument::key).collect()
    }

    pub fn key(action: BindAction) -> ConfigKey {
        let path = match action {
            BindAction::CameraTargetNext => "bind.camera.target-next",
            BindAction::CameraTargetPrevious => "bind.camera.target-previous",
            BindAction::InterfacePauseMenu => "bind.interface.pause-menu",
            BindAction::InterfaceDebugOverlay => "bind.interface.debug-overlay",
            BindAction::RenderTaaToggle => "bind.render.taa-toggle",
        };

        ConfigKey::saved(path, ConfigValueKind::KeyChord)
    }

    pub fn read(values: &ConfigValues) -> BindDocument {
        let mut keys = HashMap::new();

        for action in BindAction::ALL {
            if let Some(chord) = values.key_chord(BindDocument::key(action)) {
                keys.insert(action, chord);
            }
        }

        BindDocument { keys }
    }

    pub fn into_config(self, defaults: &BindConfig) -> BindConfig {
        let mut keys = defaults.keys.clone();
        keys.extend(self.keys);

        for action in BindAction::ALL {
            let Some(chord) = keys.get(&action) else {
                continue;
            };
            let shared = BindAction::ALL
                .into_iter()
                .filter(|other| keys.get(other) == Some(chord))
                .count();

            if shared > 1 {
                tracing::warn!("{} is bound to more than one action", chord.config_name());
            }
        }

        BindConfig { keys }
    }

    pub fn write(config: &BindConfig, layer: &mut ConfigLayer) {
        for (action, chord) in &config.keys {
            layer.set(BindDocument::key(*action).path, chord.config_name().into());
        }
    }
}
