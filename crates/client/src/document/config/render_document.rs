use super::{
    AntialiasingDocument, BloomDocument, ConfigChoice, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues,
    ExposureDocument,
};
use crate::config::RenderConfig;
use crate::renderer::{Shader, ShadingModel, Tonemapper};

pub struct RenderDocument {
    tonemapper: Option<Tonemapper>,
    stars: Option<bool>,
    bloom: BloomDocument,
    exposure: ExposureDocument,
    antialiasing: AntialiasingDocument,
    wireframe: Option<bool>,
    shader_override: Option<Option<Shader>>,
    shading_model: Option<ShadingModel>,
}

impl RenderDocument {
    pub const TONEMAPPER: ConfigKey = ConfigKey::saved(
        "render.tonemapper",
        ConfigValueKind::Choice {
            names: <Tonemapper as ConfigChoice>::names,
        },
    );
    pub const STARS: ConfigKey = ConfigKey::saved("render.stars", ConfigValueKind::Bool);
    pub const WIREFRAME: ConfigKey = ConfigKey::session("render.wireframe", ConfigValueKind::Bool);
    pub const SHADER_OVERRIDE: ConfigKey = ConfigKey::session(
        "render.shader.override",
        ConfigValueKind::Choice {
            names: <Option<Shader> as ConfigChoice>::names,
        },
    );
    pub const SHADING_MODEL: ConfigKey = ConfigKey::session(
        "debug.shading",
        ConfigValueKind::Choice {
            names: <ShadingModel as ConfigChoice>::names,
        },
    );

    pub fn read(values: &ConfigValues) -> RenderDocument {
        RenderDocument {
            tonemapper: values.choice(RenderDocument::TONEMAPPER),
            stars: values.bool(RenderDocument::STARS),
            bloom: BloomDocument::read(values),
            exposure: ExposureDocument::read(values),
            antialiasing: AntialiasingDocument::read(values),
            wireframe: values.bool(RenderDocument::WIREFRAME),
            shader_override: values.choice(RenderDocument::SHADER_OVERRIDE),
            shading_model: values.choice(RenderDocument::SHADING_MODEL),
        }
    }

    pub fn into_config(self, defaults: &RenderConfig) -> RenderConfig {
        RenderConfig {
            tonemapper: self.tonemapper.unwrap_or(defaults.tonemapper),
            stars: self.stars.unwrap_or(defaults.stars),
            bloom: self.bloom.into_config(&defaults.bloom),
            exposure: self.exposure.into_config(&defaults.exposure),
            antialiasing: self.antialiasing.into_config(&defaults.antialiasing),
            wireframe: self.wireframe.unwrap_or(defaults.wireframe),
            shader_override: self.shader_override.unwrap_or(defaults.shader_override),
            shading_model: self.shading_model.unwrap_or(defaults.shading_model),
        }
    }

    pub fn write(config: &RenderConfig, layer: &mut ConfigLayer) {
        layer.set(RenderDocument::TONEMAPPER.path, config.tonemapper.config_name().into());
        layer.set(RenderDocument::STARS.path, config.stars.into());
        BloomDocument::write(&config.bloom, layer);
        ExposureDocument::write(&config.exposure, layer);
        AntialiasingDocument::write(&config.antialiasing, layer);
        layer.set(RenderDocument::WIREFRAME.path, config.wireframe.into());
        layer.set(
            RenderDocument::SHADER_OVERRIDE.path,
            config.shader_override.config_name().into(),
        );
        layer.set(
            RenderDocument::SHADING_MODEL.path,
            config.shading_model.config_name().into(),
        );
    }
}
