use std::collections::HashMap;

use winit::keyboard::KeyCode;

use super::{
    AdaptationConfig, BindAction, BindConfig, BloomConfig, Config, DebugConfig, ExposureConfig, ExposureMode, KeyChord,
    RenderConfig, TaaConfig,
};
use crate::renderer::{ShadingModel, Tonemapper};

// Fraction of all light redistributed into the glow, like the scatter of a real lens.
const BLOOM_STRENGTH: f32 = 0.02;
const MANUAL_EV100: f32 = 15.0;
// Eyes adapt to brighter scenes far faster than to darker ones.
const DARK_TO_LIGHT_HALF_LIFE_SECONDS: f32 = 0.1;
const LIGHT_TO_DARK_HALF_LIFE_SECONDS: f32 = 1.5;

struct DefaultBind {
    action: BindAction,
    chord: KeyChord,
}

impl DefaultBind {
    fn keys() -> HashMap<BindAction, KeyChord> {
        let mut keys = HashMap::new();

        for bind in &DEFAULT_BINDS {
            keys.insert(bind.action, bind.chord);
        }

        keys
    }
}

const DEFAULT_BINDS: [DefaultBind; 5] = [
    DefaultBind {
        action: BindAction::CameraTargetNext,
        chord: KeyChord::key(KeyCode::Tab),
    },
    DefaultBind {
        action: BindAction::CameraTargetPrevious,
        chord: KeyChord::shift(KeyCode::Tab),
    },
    DefaultBind {
        action: BindAction::InterfacePauseMenu,
        chord: KeyChord::key(KeyCode::Escape),
    },
    DefaultBind {
        action: BindAction::InterfaceDebugOverlay,
        chord: KeyChord::key(KeyCode::F3),
    },
    DefaultBind {
        action: BindAction::RenderTaaToggle,
        chord: KeyChord::key(KeyCode::F1),
    },
];

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
                taa: TaaConfig { enabled: true },
            },
            bind: BindConfig {
                keys: DefaultBind::keys(),
            },
            debug: DebugConfig {
                shader_override: None,
                shading_model: ShadingModel::HapkeSol,
                wireframe: false,
            },
        }
    }
}
