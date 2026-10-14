mod animation_config;
mod antialiasing_config;
mod antialiasing_kind;
mod bind_action;
mod bind_config;
mod bloom_config;
mod default_config;
mod input_config;
mod key_chord;
mod lens_config;
mod render_config;
mod terrain_lod_config;
mod white_balance_config;

pub use animation_config::AnimationConfig;
pub use antialiasing_config::AntialiasingConfig;
pub use antialiasing_kind::AntialiasingKind;
pub use bind_action::BindAction;
pub use bind_config::BindConfig;
pub use bloom_config::BloomConfig;
pub use input_config::InputConfig;
pub use key_chord::KeyChord;
pub use lens_config::LensConfig;
pub use render_config::RenderConfig;
pub use terrain_lod_config::TerrainLodConfig;
pub use white_balance_config::WhiteBalanceConfig;

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub render: RenderConfig,
    pub bind: BindConfig,
    pub animations: AnimationConfig,
    pub input: InputConfig,
}
