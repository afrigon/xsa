use super::{AdaptationConfig, BloomConfig, Config, DebugConfig, ExposureConfig, ExposureMode, RenderConfig};
use crate::renderer::{ShadingModel, Tonemapper};

// Fraction of all light redistributed into the glow, like the scatter of a real lens.
const BLOOM_STRENGTH: f32 = 0.02;
const MANUAL_EV100: f32 = 15.0;
// Eyes adapt to brighter scenes far faster than to darker ones.
const DARK_TO_LIGHT_HALF_LIFE_SECONDS: f32 = 0.1;
const LIGHT_TO_DARK_HALF_LIFE_SECONDS: f32 = 1.5;

impl Default for Config {
    fn default() -> Self {
        Config {
            render: RenderConfig {
                tonemapper: Tonemapper::Agx,
                stars: true,
                bloom: BloomConfig {
                    enabled: true,
                    strength: BLOOM_STRENGTH,
                },
                exposure: ExposureConfig {
                    mode: ExposureMode::EyeAdaptation,
                    ev100: MANUAL_EV100,
                    compensation: 0.0,
                    adaptation: AdaptationConfig {
                        dark_to_light_half_life_seconds: DARK_TO_LIGHT_HALF_LIFE_SECONDS,
                        light_to_dark_half_life_seconds: LIGHT_TO_DARK_HALF_LIFE_SECONDS,
                    },
                },
            },
            debug: DebugConfig {
                shader_override: None,
                shading_model: ShadingModel::HapkeSol,
                wireframe: false,
            },
        }
    }
}
