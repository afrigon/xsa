use std::collections::HashMap;
use std::time::Duration;

use resource_vulkan::ResourceLimits;
use xsa_commands::logging::LoggingConfig;
use xsa_input::{InputChord, Key, Modifier};

use super::{
    AnimationConfig, AnimationGroupConfig, AntialiasingKind, BindAction, BindConfig, BloomConfig, Config,
    DebugCameraConfig, DebugConfig, InputConfig, InputSourceConfig, LensConfig, RenderConfig, ResourcesConfig,
    ServiceConfig, TerrainConfig, TerrainHeightConfig, TerrainLodConfig, VideoConfig, WhiteBalanceConfig,
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
const TERRAIN_HEIGHT_SCALE: f32 = 1.0;
// Long enough for a camera to look from one body to another and back without reloading either.
const TERRAIN_TEXTURE_RETENTION: Duration = Duration::from_secs(30);

struct DefaultBind {
    action: BindAction,
    chord: InputChord,
}

impl DefaultBind {
    fn keys() -> HashMap<BindAction, InputChord> {
        DefaultBind::all()
            .into_iter()
            .map(|bind| (bind.action, bind.chord))
            .collect()
    }

    fn all() -> [DefaultBind; 7] {
        [
            DefaultBind {
                action: BindAction::CameraTargetNext,
                chord: InputChord::key(Key::Tab),
            },
            DefaultBind {
                action: BindAction::CameraTargetPrevious,
                chord: InputChord::with_modifier(Modifier::Shift, Key::Tab),
            },
            DefaultBind {
                action: BindAction::InterfacePauseMenu,
                chord: InputChord::key(Key::Escape),
            },
            DefaultBind {
                action: BindAction::InterfaceDebugOverlay,
                chord: InputChord::key(Key::F3),
            },
            DefaultBind {
                action: BindAction::TimeFaster,
                chord: InputChord::key(Key::Period),
            },
            DefaultBind {
                action: BindAction::TimeSlower,
                chord: InputChord::key(Key::Comma),
            },
            DefaultBind {
                action: BindAction::TimeRealTime,
                chord: InputChord::key(Key::Slash),
            },
        ]
    }
}

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
                raw: InputSourceConfig { enabled: INPUT_ENABLED },
                commands: InputSourceConfig { enabled: INPUT_ENABLED },
                bind: BindConfig {
                    keys: DefaultBind::keys(),
                },
            },
            terrain: TerrainConfig {
                lod: TerrainLodConfig {
                    vertex_spacing_pixels: TERRAIN_VERTEX_SPACING_PIXELS,
                    freeze: false,
                },
                height: TerrainHeightConfig {
                    enabled: true,
                    scale: TERRAIN_HEIGHT_SCALE,
                },
                texture_retention: TERRAIN_TEXTURE_RETENTION,
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
