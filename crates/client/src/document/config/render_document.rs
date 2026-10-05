use super::{
    BloomDocument, ConfigChoice, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues, ExposureDocument, TaaDocument,
};
use crate::config::RenderConfig;
use crate::renderer::Tonemapper;

pub struct RenderDocument {
    tonemapper: Option<Tonemapper>,
    stars: Option<bool>,
    bloom: BloomDocument,
    exposure: ExposureDocument,
    taa: TaaDocument,
}

impl RenderDocument {
    pub const TONEMAPPER: ConfigKey = ConfigKey::saved(
        "render.tonemapper",
        ConfigValueKind::Choice {
            names: <Tonemapper as ConfigChoice>::names,
        },
    );
    pub const STARS: ConfigKey = ConfigKey::saved("render.stars", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> RenderDocument {
        RenderDocument {
            tonemapper: values.choice(RenderDocument::TONEMAPPER),
            stars: values.bool(RenderDocument::STARS),
            bloom: BloomDocument::read(values),
            exposure: ExposureDocument::read(values),
            taa: TaaDocument::read(values),
        }
    }

    pub fn into_config(self, defaults: &RenderConfig) -> RenderConfig {
        RenderConfig {
            tonemapper: self.tonemapper.unwrap_or(defaults.tonemapper),
            stars: self.stars.unwrap_or(defaults.stars),
            bloom: self.bloom.into_config(&defaults.bloom),
            exposure: self.exposure.into_config(&defaults.exposure),
            taa: self.taa.into_config(&defaults.taa),
        }
    }

    pub fn write(config: &RenderConfig, layer: &mut ConfigLayer) {
        layer.set(RenderDocument::TONEMAPPER.path, config.tonemapper.config_name().into());
        layer.set(RenderDocument::STARS.path, config.stars.into());
        BloomDocument::write(&config.bloom, layer);
        ExposureDocument::write(&config.exposure, layer);
        TaaDocument::write(&config.taa, layer);
    }
}
