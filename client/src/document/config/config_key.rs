use super::{AdaptationDocument, BloomDocument, ConfigValueKind, DebugDocument, ExposureDocument, RenderDocument};

#[derive(Clone, Copy)]
pub struct ConfigKey {
    pub path: &'static str,
    pub kind: ConfigValueKind,
    pub persisted: bool,
}

impl ConfigKey {
    pub const ALL: &[ConfigKey] = &[
        RenderDocument::TONEMAPPER,
        RenderDocument::STARS,
        BloomDocument::ENABLED,
        BloomDocument::STRENGTH,
        ExposureDocument::MODE,
        ExposureDocument::EV100,
        ExposureDocument::COMPENSATION,
        AdaptationDocument::DARK_TO_LIGHT,
        AdaptationDocument::LIGHT_TO_DARK,
        DebugDocument::SHADER,
        DebugDocument::SHADING,
        DebugDocument::WIREFRAME,
    ];

    pub fn find(path: &str) -> Option<ConfigKey> {
        ConfigKey::ALL.iter().copied().find(|key| key.path == path)
    }

    pub const fn saved(path: &'static str, kind: ConfigValueKind) -> ConfigKey {
        ConfigKey {
            path,
            kind,
            persisted: true,
        }
    }

    pub const fn session(path: &'static str, kind: ConfigValueKind) -> ConfigKey {
        ConfigKey {
            path,
            kind,
            persisted: false,
        }
    }
}
