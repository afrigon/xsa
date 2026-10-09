use super::{ConfigChoice, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::DebugConfig;
use crate::renderer::{Shader, ShadingModel};

pub struct DebugDocument {
    shader: Option<Option<Shader>>,
    shading: Option<ShadingModel>,
    wireframe: Option<bool>,
    animation_duration_scale: Option<f32>,
}

impl DebugDocument {
    pub const SHADER: ConfigKey = ConfigKey::session(
        "debug.shader",
        ConfigValueKind::Choice {
            names: <Option<Shader> as ConfigChoice>::names,
        },
    );
    pub const SHADING: ConfigKey = ConfigKey::session(
        "debug.shading",
        ConfigValueKind::Choice {
            names: <ShadingModel as ConfigChoice>::names,
        },
    );
    pub const WIREFRAME: ConfigKey = ConfigKey::session("debug.wireframe", ConfigValueKind::Bool);
    pub const ANIMATION_DURATION_SCALE: ConfigKey =
        ConfigKey::session("debug.animation-duration-scale", ConfigValueKind::Number);

    pub fn read(values: &ConfigValues) -> DebugDocument {
        DebugDocument {
            shader: values.choice(DebugDocument::SHADER),
            shading: values.choice(DebugDocument::SHADING),
            wireframe: values.bool(DebugDocument::WIREFRAME),
            animation_duration_scale: values.number(DebugDocument::ANIMATION_DURATION_SCALE),
        }
    }

    pub fn into_config(self, defaults: &DebugConfig) -> DebugConfig {
        DebugConfig {
            shader_override: self.shader.unwrap_or(defaults.shader_override),
            shading_model: self.shading.unwrap_or(defaults.shading_model),
            wireframe: self.wireframe.unwrap_or(defaults.wireframe),
            animation_duration_scale: self
                .animation_duration_scale
                .unwrap_or(defaults.animation_duration_scale),
        }
    }

    pub fn write(config: &DebugConfig, layer: &mut ConfigLayer) {
        layer.set(DebugDocument::SHADER.path, config.shader_override.config_name().into());
        layer.set(DebugDocument::SHADING.path, config.shading_model.config_name().into());
        layer.set(DebugDocument::WIREFRAME.path, config.wireframe.into());
        layer.set(
            DebugDocument::ANIMATION_DURATION_SCALE.path,
            ConfigValueKind::number(config.animation_duration_scale),
        );
    }
}
