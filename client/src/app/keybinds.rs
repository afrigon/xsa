use winit::keyboard::KeyCode;

use crate::input::Input;
use crate::renderer::{MaterialHandle, Renderer, Shader};

const EXPOSURE_STEP_STOPS: f32 = 1.0 / 3.0;
const BLOOM_STRENGTH_STEP_STOPS: f32 = 0.5;

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
    pub fn poll(&self, input: &Input, renderer: &mut Renderer, skybox: Option<MaterialHandle>) {
        for shortcut in &SHADER_SHORTCUTS {
            if input.was_pressed(shortcut.key) {
                renderer.set_shader_override(shortcut.shader);
                println!("view: {}", shortcut.shader.map_or("shaded", Shader::path));
            }
        }

        if input.was_pressed(KeyCode::Backquote) {
            match renderer.toggle_wireframe() {
                Some(enabled) => println!("wireframe: {}", Keybinds::on_off(enabled)),
                None => println!("wireframe: unsupported by this device"),
            }
        }

        if input.was_pressed(KeyCode::F2) {
            let scene = renderer.scene_mut();
            scene.skybox = if scene.skybox.is_some() { None } else { skybox };
            println!("skybox: {}", Keybinds::on_off(scene.skybox.is_some()));
        }

        if input.was_pressed(KeyCode::Minus) {
            renderer.adjust_exposure(-EXPOSURE_STEP_STOPS);
            Keybinds::print_exposure(renderer);
        }

        if input.was_pressed(KeyCode::Equal) {
            renderer.adjust_exposure(EXPOSURE_STEP_STOPS);
            Keybinds::print_exposure(renderer);
        }

        if input.was_pressed(KeyCode::F4) {
            renderer.toggle_auto_exposure();
            Keybinds::print_exposure(renderer);
        }

        if input.was_pressed(KeyCode::F5) {
            let enabled = renderer.toggle_bloom();
            println!("bloom: {}", Keybinds::on_off(enabled));
        }

        if input.was_pressed(KeyCode::BracketLeft) {
            println!(
                "bloom strength: {:.4}",
                renderer.adjust_bloom_strength(-BLOOM_STRENGTH_STEP_STOPS)
            );
        }

        if input.was_pressed(KeyCode::BracketRight) {
            println!(
                "bloom strength: {:.4}",
                renderer.adjust_bloom_strength(BLOOM_STRENGTH_STEP_STOPS)
            );
        }

        if input.was_pressed(KeyCode::F7) {
            println!("shading: {}", renderer.cycle_shading_model().name());
        }

        if input.was_pressed(KeyCode::F3) {
            println!("tonemapper: {}", renderer.cycle_tonemapper().name());
        }
    }

    fn on_off(enabled: bool) -> &'static str {
        if enabled { "on" } else { "off" }
    }

    fn print_exposure(renderer: &Renderer) {
        if renderer.auto_exposure_enabled() {
            println!(
                "exposure: auto, EV100 {:.2}, compensation {:+.2}",
                renderer.exposure_ev100(),
                renderer.exposure_compensation()
            );
        } else {
            println!("exposure: manual, EV100 {:.2}", renderer.exposure_ev100());
        }
    }
}
