use super::{
    AdaptationDocument, BindDocument, BloomDocument, ConfigValueKind, DebugDocument, ExposureDocument, RenderDocument,
    TaaDocument,
};

#[derive(Clone, Copy)]
pub struct ConfigKey {
    pub path: &'static str,
    pub kind: ConfigValueKind,
    pub persisted: bool,
}

impl ConfigKey {
    const SETTINGS: &[ConfigKey] = &[
        RenderDocument::TONEMAPPER,
        RenderDocument::STARS,
        BloomDocument::ENABLED,
        BloomDocument::STRENGTH,
        ExposureDocument::MODE,
        ExposureDocument::EV100,
        ExposureDocument::COMPENSATION,
        AdaptationDocument::DARK_TO_LIGHT,
        AdaptationDocument::LIGHT_TO_DARK,
        TaaDocument::ENABLED,
        DebugDocument::SHADER,
        DebugDocument::SHADING,
        DebugDocument::WIREFRAME,
    ];

    pub fn all() -> Vec<ConfigKey> {
        let mut keys = ConfigKey::SETTINGS.to_vec();
        keys.extend(BindDocument::keys());

        keys
    }

    pub fn find(path: &str) -> Option<ConfigKey> {
        ConfigKey::all().into_iter().find(|key| key.path == path)
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
