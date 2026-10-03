use winit::keyboard::KeyCode;

use crate::config::{Config, ExposureMode};
use crate::input::Input;
use crate::renderer::Shader;

struct ShaderShortcut {
    key: KeyCode,
    shader: Option<Shader>,
}

const SHADER_SHORTCUTS: [ShaderShortcut; 5] = [
    ShaderShortcut {
        key: KeyCode::Digit1,
        shader: None,
    },
    ShaderShortcut {
        key: KeyCode::Digit2,
        shader: Some(Shader::Normals),
    },
    ShaderShortcut {
        key: KeyCode::Digit3,
        shader: Some(Shader::Depth),
    },
    ShaderShortcut {
        key: KeyCode::Digit4,
        shader: Some(Shader::Triangles),
    },
    ShaderShortcut {
        key: KeyCode::Digit5,
        shader: Some(Shader::Lighting),
    },
];

pub(super) struct Keybinds;

impl Keybinds {
    pub fn poll_config_changes(&self, input: &Input, config: &mut Config, supports_wireframe: bool) -> bool {
        let before = config.clone();

        for shortcut in &SHADER_SHORTCUTS {
            if input.was_pressed(shortcut.key) {
                config.debug.shader_override = shortcut.shader;
                println!("view: {}", shortcut.shader.map_or("shaded", Shader::path));
            }
        }

        if input.was_pressed(KeyCode::Backquote) {
            if supports_wireframe {
                config.debug.wireframe = !config.debug.wireframe;
                println!("wireframe: {}", Keybinds::on_off(config.debug.wireframe));
            } else {
                println!("wireframe: unsupported by this device");
            }
        }

        if input.was_pressed(KeyCode::F2) {
            config.render.stars = !config.render.stars;
            println!("skybox: {}", Keybinds::on_off(config.render.stars));
        }

        if input.was_pressed(KeyCode::F4) {
            let exposure = &mut config.render.exposure;
            exposure.mode = match exposure.mode {
                ExposureMode::EyeAdaptation => ExposureMode::Manual,
                ExposureMode::Manual => ExposureMode::EyeAdaptation,
            };
            println!("exposure: {:?}", exposure.mode);
        }

        if input.was_pressed(KeyCode::F5) {
            config.render.bloom.enabled = !config.render.bloom.enabled;
            println!("bloom: {}", Keybinds::on_off(config.render.bloom.enabled));
        }

        if input.was_pressed(KeyCode::F7) {
            config.debug.shading_model = config.debug.shading_model.next();
            println!("shading: {}", config.debug.shading_model.name());
        }

        if input.was_pressed(KeyCode::F3) {
            config.render.tonemapper = config.render.tonemapper.next();
            println!("tonemapper: {}", config.render.tonemapper.name());
        }

        *config != before
    }

    fn on_off(enabled: bool) -> &'static str {
        if enabled { "on" } else { "off" }
    }
}
