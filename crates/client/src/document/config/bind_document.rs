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
            BindAction::CameraDebugToggle => "bind.camera.debug-toggle",
            BindAction::CameraTargetNext => "bind.camera.target-next",
            BindAction::CameraTargetPrevious => "bind.camera.target-previous",
            BindAction::CameraReleaseMouse => "bind.camera.release-mouse",
            BindAction::RenderStarsToggle => "bind.render.stars-toggle",
            BindAction::RenderTonemapperNext => "bind.render.tonemapper-next",
            BindAction::RenderExposureModeToggle => "bind.render.exposure-mode-toggle",
            BindAction::RenderBloomToggle => "bind.render.bloom-toggle",
            BindAction::DebugShadingNext => "bind.debug.shading-next",
            BindAction::DebugWireframeToggle => "bind.debug.wireframe-toggle",
            BindAction::DebugShaderLit => "bind.debug.shader-lit",
            BindAction::DebugShaderNormals => "bind.debug.shader-normals",
            BindAction::DebugShaderDepth => "bind.debug.shader-depth",
            BindAction::DebugShaderTriangles => "bind.debug.shader-triangles",
            BindAction::DebugShaderLighting => "bind.debug.shader-lighting",
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
