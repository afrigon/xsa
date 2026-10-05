use super::{ConfigChoice, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::{AntialiasingConfig, AntialiasingKind};

pub struct AntialiasingDocument {
    kind: Option<Option<AntialiasingKind>>,
}

impl AntialiasingDocument {
    pub const KIND: ConfigKey = ConfigKey::saved(
        "render.antialiasing.kind",
        ConfigValueKind::Choice {
            names: <Option<AntialiasingKind> as ConfigChoice>::names,
        },
    );

    pub fn read(values: &ConfigValues) -> AntialiasingDocument {
        AntialiasingDocument {
            kind: values.choice(AntialiasingDocument::KIND),
        }
    }

    pub fn into_config(self, defaults: &AntialiasingConfig) -> AntialiasingConfig {
        AntialiasingConfig {
            kind: self.kind.unwrap_or(defaults.kind),
        }
    }

    pub fn write(config: &AntialiasingConfig, layer: &mut ConfigLayer) {
        layer.set(AntialiasingDocument::KIND.path, config.kind.config_name().into());
    }
}
