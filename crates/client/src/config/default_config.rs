use std::collections::HashMap;

use winit::keyboard::KeyCode;

use super::{
    AnimationConfig, AntialiasingConfig, AntialiasingKind, BindAction, BindConfig, BloomConfig, Config, ExposureConfig,
    ExposureMode, KeyChord, RenderConfig,
};
use crate::renderer::{ShadingModel, Tonemapper};

// Fraction of all light redistributed into the glow, like the scatter of a real lens.
const BLOOM_STRENGTH: f32 = 0.02;
const MANUAL_EV100: f32 = 15.0;
const ANIMATION_DURATION_SCALE: f32 = 1.0;

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

const DEFAULT_BINDS: [DefaultBind; 4] = [
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
                },
                antialiasing: AntialiasingConfig {
                    kind: Some(AntialiasingKind::Taa),
                },
                wireframe: false,
                shader_override: None,
                shading_model: ShadingModel::HapkeSol,
            },
            bind: BindConfig {
                keys: DefaultBind::keys(),
            },
            animations: AnimationConfig {
                duration_scale: ANIMATION_DURATION_SCALE,
            },
        }
    }
}
