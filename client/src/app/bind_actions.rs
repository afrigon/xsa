use xsa_commands::command::{
    CameraMode as CommandCameraMode, CameraModeCommand, CameraTargetCommand, ConfigSetCommand, ConfigToggleCommand,
};
use xsa_commands::value::Target;

use super::App;
use super::client_command_handler::ClientCommandHandler;
use crate::camera::CameraMode;
use crate::config::BindAction;
use crate::document::{BloomDocument, ConfigChoice, ConfigKey, DebugDocument, ExposureDocument, RenderDocument};
use crate::renderer::Shader;

impl App {
    pub(super) fn run_bind_action(&mut self, action: BindAction) {
        let result = match action {
            BindAction::CameraDebugToggle => {
                let mode = match self.cameras.toggled_mode() {
                    CameraMode::Orbit => CommandCameraMode::Target,
                    CameraMode::Debug => CommandCameraMode::Debug,
                };
                CameraModeCommand { mode }.run(self)
            }
            BindAction::CameraTargetNext => App::target(Target::Next).run(self),
            BindAction::CameraTargetPrevious => App::target(Target::Previous).run(self),
            BindAction::CameraReleaseMouse => {
                self.set_mouse_captured(false);

                return;
            }
            BindAction::RenderStarsToggle => App::toggle(RenderDocument::STARS).run(self),
            BindAction::RenderTonemapperNext => App::toggle(RenderDocument::TONEMAPPER).run(self),
            BindAction::RenderExposureModeToggle => App::toggle(ExposureDocument::MODE).run(self),
            BindAction::RenderBloomToggle => App::toggle(BloomDocument::ENABLED).run(self),
            BindAction::DebugShadingNext => App::toggle(DebugDocument::SHADING).run(self),
            BindAction::DebugWireframeToggle => App::toggle(DebugDocument::WIREFRAME).run(self),
            BindAction::DebugShaderLit => App::shader(None).run(self),
            BindAction::DebugShaderNormals => App::shader(Some(Shader::Normals)).run(self),
            BindAction::DebugShaderDepth => App::shader(Some(Shader::Depth)).run(self),
            BindAction::DebugShaderTriangles => App::shader(Some(Shader::Triangles)).run(self),
            BindAction::DebugShaderLighting => App::shader(Some(Shader::Lighting)).run(self),
        };

        match result {
            Ok(description) => tracing::info!("{description}"),
            Err(err) => tracing::warn!("{err:#}"),
        }
    }

    fn target(target: Target) -> CameraTargetCommand {
        CameraTargetCommand {
            target,
            category: None,
            distance: None,
            pitch: None,
            yaw: None,
        }
    }

    fn toggle(key: ConfigKey) -> ConfigToggleCommand {
        ConfigToggleCommand {
            key: key.path.to_string(),
            save: false,
        }
    }

    fn shader(shader: Option<Shader>) -> ConfigSetCommand {
        ConfigSetCommand {
            key: DebugDocument::SHADER.path.to_string(),
            value: shader.config_name().to_string(),
            save: false,
        }
    }
}
