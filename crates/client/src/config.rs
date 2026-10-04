mod adaptation_config;
mod bind_action;
mod bind_config;
mod bloom_config;
mod debug_config;
mod default_config;
mod exposure_config;
mod exposure_mode;
mod key_chord;
mod render_config;

pub use adaptation_config::AdaptationConfig;
pub use bind_action::BindAction;
pub use bind_config::BindConfig;
pub use bloom_config::BloomConfig;
pub use debug_config::DebugConfig;
pub use exposure_config::ExposureConfig;
pub use exposure_mode::ExposureMode;
pub use key_chord::KeyChord;
pub use render_config::RenderConfig;

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub render: RenderConfig,
    pub bind: BindConfig,
    pub debug: DebugConfig,
}
