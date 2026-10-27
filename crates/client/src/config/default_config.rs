use std::time::Duration;

use resource_vulkan::ResourceLimits;
use xsa_commands::logging::LoggingConfig;
use xsa_config::ConfigValue;
use xsa_input::InputChord;

use super::{
    AnimationConfig, AnimationGroupConfig, AntialiasingKind, Bind, BindCommand, BindConfig, BloomConfig, CameraConfig,
    Config, DebugCameraConfig, DebugConfig, FadeConfig, InputConfig, InputSourceConfig, LensConfig, NamedBind,
    RenderConfig, ResourcesConfig, ServiceConfig, TerrainConfig, TerrainHeightConfig, TerrainLodConfig,
    TerrainScatterConfig, VideoConfig, WhiteBalanceConfig,
};

use crate::renderer::Tonemapper;

// Fraction of all light redistributed into the glow, like the scatter of a real lens.
const BLOOM_STRENGTH: f32 = 0.02;
const RUNNING_ANIMATIONS: AnimationGroupConfig = AnimationGroupConfig {
    enabled: true,
    duration_scale: 1.0,
};
const INPUT_ENABLED: bool = true;
const CAMERA_COLLISION: bool = true;
const DEBUG_CAMERA_SPEED: f32 = 2_000_000.0;
const CAMERA_FIELD_OF_VIEW_DEGREES: f32 = 100.0;
const CAMERA_ZOOM: f32 = 1.0;
// A starting point to tune from, not a measured optimum.
const TERRAIN_VERTEX_SPACING_PIXELS: f32 = 16.0;
const TERRAIN_HEIGHT_SCALE: f32 = 1.0;
// Long enough for a camera to look from one body to another and back without reloading either.
const TERRAIN_TEXTURE_RETENTION: Duration = Duration::from_secs(30);

// A bind the game starts with: `down` runs as its key goes down, `up`, if any, as it is released.
struct DefaultBind {
    category: &'static str,
    name: &'static str,
    key: &'static str,
    down: &'static str,
    up: Option<&'static str>,
}

impl DefaultBind {
    const ALL: [DefaultBind; 8] = [
        DefaultBind::new("camera", "target-next", "tab", "camera target --next"),
        DefaultBind::new("camera", "target-previous", "shift+tab", "camera target --previous"),
        DefaultBind {
            up: Some("config unset camera.zoom"),
            ..DefaultBind::new("camera", "zoom", "c", "config set camera.zoom 4")
        },
        DefaultBind::new("interface", "pause-menu", "escape", "ui toggle pause-menu"),
        DefaultBind::new("interface", "debug-overlay", "f3", "ui toggle debug-overlay"),
        DefaultBind::new("time", "faster", ".", "time faster"),
        DefaultBind::new("time", "slower", ",", "time slower"),
        DefaultBind::new("time", "real-time", "/", "time resume"),
    ];

    const fn new(category: &'static str, name: &'static str, key: &'static str, down: &'static str) -> DefaultBind {
        DefaultBind {
            category,
            name,
            key,
            down,
            up: None,
        }
    }

    fn binds() -> Vec<NamedBind> {
        let command = |text: &str| BindCommand::parse(text).expect("default binds run valid commands");

        DefaultBind::ALL
            .into_iter()
            .map(|default| NamedBind {
                category: default.category,
                name: default.name,
                bind: Bind {
                    key: InputChord::from_name(default.key).expect("default binds name valid keys"),
                    down: command(default.down),
                    up: default.up.map(command),
                },
            })
            .collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            camera: CameraConfig {
                fov: CAMERA_FIELD_OF_VIEW_DEGREES,
                zoom: CAMERA_ZOOM,
                collision: CAMERA_COLLISION,
                debug: DebugCameraConfig {
                    speed: DEBUG_CAMERA_SPEED,
                },
            },
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
                fade: FadeConfig { enabled: true },
            },
            input: InputConfig {
                enabled: INPUT_ENABLED,
                raw: InputSourceConfig { enabled: INPUT_ENABLED },
                commands: InputSourceConfig { enabled: INPUT_ENABLED },
                bind: BindConfig {
                    binds: DefaultBind::binds(),
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
                scatter: TerrainScatterConfig { enabled: true },
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
            debug: DebugConfig { wireframe: false },
            logging: LoggingConfig::default(),
        }
    }
}
