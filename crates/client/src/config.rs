mod animation_config;
mod animation_group_config;
mod antialiasing_kind;
mod antialiasing_kind_choice;
mod bind_action;
mod bind_config;
mod bloom_config;
mod debug_camera_config;
mod debug_config;
mod debug_view_choice;
mod default_config;
mod input_config;
mod input_source_config;
mod lens_config;
mod render_config;
mod resources_config;
mod service_config;
mod surface_debug_view_choice;
mod terrain_config;
mod terrain_debug_view_choice;
mod terrain_height_config;
mod terrain_lod_config;
mod tonemapper_choice;
mod video_config;
mod white_balance_config;

pub use animation_config::AnimationConfig;
pub use animation_group_config::AnimationGroupConfig;
pub use antialiasing_kind::AntialiasingKind;
pub use bind_action::BindAction;
pub use bind_config::BindConfig;
pub use bloom_config::BloomConfig;
pub use debug_camera_config::DebugCameraConfig;
pub use debug_config::DebugConfig;
pub use input_config::InputConfig;
pub use input_source_config::InputSourceConfig;
pub use lens_config::LensConfig;
pub use render_config::RenderConfig;
pub use resources_config::ResourcesConfig;
pub use service_config::ServiceConfig;
pub use terrain_config::TerrainConfig;
pub use terrain_height_config::TerrainHeightConfig;
pub use terrain_lod_config::TerrainLodConfig;
pub use video_config::VideoConfig;
pub use white_balance_config::WhiteBalanceConfig;

use std::env;
use std::path::PathBuf;

use anyhow::Context;
use xsa_commands::logging::LoggingConfig;
use xsa_config::{ConfigSection, ConfigVisitor};

const DIRECTORY_NAME: &str = "xsa";
const FILE_NAME: &str = "config.kdl";

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub video: VideoConfig,
    pub input: InputConfig,
    pub terrain: TerrainConfig,
    pub animation: AnimationConfig,
    pub service: ServiceConfig,
    pub render: RenderConfig,
    pub debug: DebugConfig,
    pub logging: LoggingConfig,
}

impl Config {
    pub fn default_path() -> anyhow::Result<PathBuf> {
        let directory = match env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
            Some(directory) => PathBuf::from(directory),
            None if cfg!(windows) => PathBuf::from(env::var_os("APPDATA").context("APPDATA is not set")?),
            None => PathBuf::from(env::var_os("HOME").context("HOME is not set")?).join(".config"),
        };

        Ok(directory.join(DIRECTORY_NAME).join(FILE_NAME))
    }
}

impl ConfigSection for Config {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.section("video", &mut self.video);
        visitor.section("input", &mut self.input);
        visitor.section("terrain", &mut self.terrain);
        visitor.section("animation", &mut self.animation);
        visitor.section("service", &mut self.service);
        visitor.section("render", &mut self.render);
        visitor.section("debug", &mut self.debug);
        self.logging.visit(visitor);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use xsa_config::ConfigStore;

    use super::*;

    fn store(name: &str) -> ConfigStore<Config> {
        let path = env::temp_dir().join(format!("xsa-client-config-{name}-{}.kdl", std::process::id()));
        let _ = fs::remove_file(&path);

        ConfigStore::load(path).unwrap()
    }

    #[test]
    fn the_tree_holds_every_key_once() {
        let store = store("tree");
        let paths: Vec<&str> = store.keys().iter().map(|key| key.path.as_str()).collect();

        for path in [
            "video.tonemapper",
            "video.bloom.strength",
            "video.white-balance.enabled",
            "input.enabled",
            "input.raw.enabled",
            "input.commands.enabled",
            "input.bind.camera.target-next",
            "terrain.lod.vertex-spacing",
            "terrain.texture-retention",
            "animation.duration-scale",
            "animation.camera.enabled",
            "service.resources.video-memory.max-usage",
            "service.resources.workers.max-count",
            "service.resources.read-delay",
            "render.shader.terrain.override",
            "debug.wireframe",
            "logging.filter",
            "logging.shell.ipc.input.enabled",
            "logging.shell.script.enabled",
        ] {
            assert!(paths.contains(&path), "{path} is missing from {paths:?}");
        }

        assert_eq!(store.config(), &Config::default());
    }

    #[test]
    fn binds_with_symbol_keys_survive_a_save() {
        let mut store = store("binds");
        let binds = [
            "input.bind.camera.target-next",
            "input.bind.camera.target-previous",
            "input.bind.interface.pause-menu",
            "input.bind.interface.debug-overlay",
        ];
        let keys = ["\\", "=", "[", "ctrl+'"];

        for (bind, key) in binds.iter().zip(keys) {
            store.set(bind, key, true).unwrap();
        }

        let reloaded = ConfigStore::<Config>::load(store.path().to_path_buf()).unwrap();

        for (bind, key) in binds.iter().zip(keys) {
            assert_eq!(reloaded.get(bind).unwrap(), key);
        }

        assert_eq!(reloaded.config().input.bind, store.config().input.bind);
        let _ = fs::remove_file(store.path());
    }

    #[test]
    fn an_animation_group_runs_at_the_product_of_its_scales_only_when_enabled() {
        let mut store = store("animation");
        store.set("animation.duration-scale", "0.5", false).unwrap();
        store.set("animation.ui.duration-scale", "4", false).unwrap();
        assert_eq!(store.config().animation.ui_duration_scale(), 2.0);
        assert_eq!(store.config().animation.camera_duration_scale(), 0.5);

        store.set("animation.camera.enabled", "false", false).unwrap();
        assert_eq!(store.config().animation.camera_duration_scale(), 0.0);
        store.set("animation.enabled", "false", false).unwrap();
        assert_eq!(store.config().animation.ui_duration_scale(), 0.0);
    }
}
