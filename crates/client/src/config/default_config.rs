use std::collections::HashMap;

use resource_vulkan::ResourceLimits;
use winit::keyboard::KeyCode;

use super::{
    AnimationConfig, AntialiasingConfig, AntialiasingKind, BindAction, BindConfig, BloomConfig, Config, InputConfig,
    KeyChord, LensConfig, RenderConfig, TerrainLodConfig, WhiteBalanceConfig,
};

use crate::renderer::Tonemapper;

// Fraction of all light redistributed into the glow, like the scatter of a real lens.
const BLOOM_STRENGTH: f32 = 0.02;
const ANIMATION_DURATION_SCALE: f32 = 1.0;
const INPUT_ENABLED: bool = true;
// A starting point to tune from, not a measured optimum.
const TERRAIN_VERTEX_SPACING_PIXELS: f32 = 16.0;

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

const DEFAULT_BINDS: [DefaultBind; 7] = [
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
        action: BindAction::TimeFaster,
        chord: KeyChord::key(KeyCode::Period),
    },
    DefaultBind {
        action: BindAction::TimeSlower,
        chord: KeyChord::key(KeyCode::Comma),
    },
    DefaultBind {
        action: BindAction::TimeRealTime,
        chord: KeyChord::key(KeyCode::Slash),
    },
];

impl Default for Config {
    fn default() -> Self {
        Config {
            render: RenderConfig {
                tonemapper: Some(Tonemapper::Agx),
                stars: true,
                bloom: BloomConfig {
                    enabled: true,
                    strength: BLOOM_STRENGTH,
                },
                lens: LensConfig { enabled: true },
                white_balance: WhiteBalanceConfig { enabled: true },
                antialiasing: AntialiasingConfig {
                    kind: Some(AntialiasingKind::Taa),
                },
                wireframe: false,
                shader_override: None,
                terrain_lod: TerrainLodConfig {
                    vertex_spacing_pixels: TERRAIN_VERTEX_SPACING_PIXELS,
                    freeze: false,
                },
            },
            bind: BindConfig {
                keys: DefaultBind::keys(),
            },
            animations: AnimationConfig {
                duration_scale: ANIMATION_DURATION_SCALE,
            },
            input: InputConfig { enabled: INPUT_ENABLED },
            resources: ResourceLimits::default(),
        }
    }
}
