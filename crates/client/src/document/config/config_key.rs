use super::{
    AnimationDocument, AntialiasingDocument, BindDocument, BloomDocument, ConfigValueKind, InputDocument, LensDocument,
    RenderDocument, TerrainLodDocument, WhiteBalanceDocument,
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
        LensDocument::ENABLED,
        WhiteBalanceDocument::ENABLED,
        AntialiasingDocument::KIND,
        RenderDocument::WIREFRAME,
        RenderDocument::SHADER_OVERRIDE,
        RenderDocument::SHADING_MODEL,
        TerrainLodDocument::VERTEX_SPACING,
        TerrainLodDocument::FREEZE,
        AnimationDocument::DURATION_SCALE,
        InputDocument::ENABLED,
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
