use std::collections::HashMap;

use resource_vulkan::ResourceLimits;
use winit::keyboard::KeyCode;
use xsa_commands::logging::LoggingConfig;

use super::{
    AnimationConfig, AnimationGroupConfig, AntialiasingKind, BindAction, BindConfig, BloomConfig, Config,
    DebugCameraConfig, DebugConfig, InputConfig, KeyChord, LensConfig, RenderConfig, ResourcesConfig, ServiceConfig,
    TerrainConfig, TerrainLodConfig, TerrainStreamingConfig, VideoConfig, WhiteBalanceConfig,
};

use crate::renderer::Tonemapper;

// Fraction of all light redistributed into the glow, like the scatter of a real lens.
const BLOOM_STRENGTH: f32 = 0.02;
const RUNNING_ANIMATIONS: AnimationGroupConfig = AnimationGroupConfig {
    enabled: true,
    duration_scale: 1.0,
};
const INPUT_ENABLED: bool = true;
const DEBUG_CAMERA_SPEED: f32 = 2_000_000.0;
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
            video: VideoConfig {
                tonemapper: Some(Tonemapper::Agx),
                stars: true,
                antialiasing: Some(AntialiasingKind::Taa),
                bloom: BloomConfig {
                    enabled: true,
                    strength: BLOOM_STRENGTH,
                },
                lens: LensConfig { enabled: true },
                white_balance: WhiteBalanceConfig { enabled: true },
            },
            input: InputConfig {
                enabled: INPUT_ENABLED,
                bind: BindConfig {
                    keys: DefaultBind::keys(),
                },
            },
            terrain: TerrainConfig {
                lod: TerrainLodConfig {
                    vertex_spacing_pixels: TERRAIN_VERTEX_SPACING_PIXELS,
                    freeze: false,
                },
                streaming: TerrainStreamingConfig { freeze: false },
            },
            animation: AnimationConfig {
                all: RUNNING_ANIMATIONS,
                camera: RUNNING_ANIMATIONS,
                ui: RUNNING_ANIMATIONS,
                time: RUNNING_ANIMATIONS,
            },
            service: ServiceConfig {
                resources: ResourcesConfig::from_limits(ResourceLimits::default()),
            },
            render: RenderConfig {
                shader_override: None,
                terrain_shader_override: None,
                surface_shader_override: None,
            },
            debug: DebugConfig {
                wireframe: false,
                camera: DebugCameraConfig {
                    speed: DEBUG_CAMERA_SPEED,
                },
            },
            logging: LoggingConfig::default(),
        }
    }
}
