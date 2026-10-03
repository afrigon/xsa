mod camera_controller;
mod client_command_handler;
mod client_world;
mod command_handlers;
mod keybinds;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, bail};
use glam::DVec2;
use tokio::sync::mpsc::UnboundedReceiver;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use xsa_commands::command::{CameraTargetCommand, ClientCommand};
use xsa_commands::completion::{CompletionCandidate, CompletionKind};
use xsa_commands::router::{CommandExecutor, CommandInvocation, CommandRouter};
use xsa_commands::value::Target;
use xsa_packs::PackStack;
use xsa_proto::event::{ServerEvent, WorldState};
use xsa_proto::message::{ClientMessage, Leave};
use xsa_proto::session::ServerSession;

use crate::camera::CameraMode;
use crate::config::Config;
use crate::document::{ConfigDocument, ConfigKey};
use crate::input::Input;
use crate::renderer::{Renderer, ShaderBinaries};
use camera_controller::CameraController;
use client_command_handler::ClientCommandHandler;
use client_world::ClientWorld;
use keybinds::Keybinds;

const APP_ID: &str = "xsa";
const BASE_PACK: &str = "base";

pub struct App {
    renderer: Option<Renderer>,
    window: Option<Window>,
    error: Option<anyhow::Error>,
    session: ServerSession,
    router: CommandRouter,
    invocations: UnboundedReceiver<CommandInvocation>,
    exit_requested: bool,
    packs_directory: PathBuf,
    world: Option<ClientWorld>,
    input: Input,
    cameras: CameraController,
    keybinds: Keybinds,
    config_document: ConfigDocument,
    config: Config,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

impl App {
    pub fn new(
        session: ServerSession,
        invocations: UnboundedReceiver<CommandInvocation>,
        packs_directory: PathBuf,
        config_document: ConfigDocument,
    ) -> Self {
        Self {
            renderer: None,
            window: None,
            error: None,
            session,
            router: CommandRouter::default(),
            invocations,
            exit_requested: false,
            packs_directory,
            world: None,
            input: Input::default(),
            cameras: CameraController::new(),
            keybinds: Keybinds,
            config: config_document.config(),
            config_document,
            mouse_captured: false,
            last_frame: None,
        }
    }

    pub fn into_result(self) -> anyhow::Result<()> {
        self.error.map_or(Ok(()), Err)
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = event_loop.create_window(App::window_attributes())?;
        let base = PackStack::load(&self.packs_directory, &[BASE_PACK.to_string()])?;
        let shaders = ShaderBinaries::load(&base)?;
        self.renderer = Some(Renderer::new(&window, &shaders, &self.config)?);
        window.request_redraw();
        self.window = Some(window);

        Ok(())
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let delta_seconds = self.last_frame.map_or(0.0, |last| (now - last).as_secs_f64());
        self.last_frame = Some(now);

        self.update(delta_seconds)?;
        self.draw()?;
        self.input.end_frame();

        if let Some(window) = &self.window {
            window.request_redraw();
        }

        Ok(())
    }

    fn update(&mut self, delta_seconds: f64) -> anyhow::Result<()> {
        self.poll_connection()?;
        self.poll_invocations();
        self.poll_input();

        if let Some(world) = &mut self.world {
            if let Some(state) = self.session.state() {
                world.advance(state.time());
            }

            self.cameras.update(world, &self.input, delta_seconds);
        }

        Ok(())
    }

    fn poll_connection(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.session.poll()? {
            self.router.handle_event(&event, &self.session);

            match event {
                ServerEvent::JoinAccepted(accepted) => self
                    .join(&accepted.state)
                    .with_context(|| format!("joining simulation {}", accepted.state.simulation))?,
                ServerEvent::JoinDenied(denied) => bail!("the server refused to let us join: {}", denied.reason),
                ServerEvent::PlayerJoined(_)
                | ServerEvent::PlayerLeft(_)
                | ServerEvent::TimeChanged(_)
                | ServerEvent::Tick(_)
                | ServerEvent::Reply(_) => {}
            }
        }

        Ok(())
    }

    fn poll_invocations(&mut self) {
        let mut router = std::mem::take(&mut self.router);
        router.resume(self);

        while let Ok(invocation) = self.invocations.try_recv() {
            router.handle(invocation, self);
        }

        self.router = router;
    }

    fn join(&mut self, state: &WorldState) -> anyhow::Result<()> {
        let renderer = self.renderer.as_mut().context("the renderer is not ready")?;
        let world = ClientWorld::load(&self.packs_directory, state, renderer)?;
        self.cameras.focus(&world);
        self.world = Some(world);
        self.apply_config();

        Ok(())
    }

    fn poll_input(&mut self) {
        if self.input.was_pressed(KeyCode::Escape) {
            self.set_mouse_captured(false);
        }

        if self.input.was_pressed(KeyCode::F1) {
            self.set_camera_mode(self.cameras.toggled_mode());
            println!("camera: {}", self.cameras.mode().name());
        }

        if self.cameras.mode() == CameraMode::Orbit && self.input.was_pressed(KeyCode::Tab) {
            let backwards = self.input.is_held(KeyCode::ShiftLeft) || self.input.is_held(KeyCode::ShiftRight);
            self.cycle_target(if backwards { Target::Previous } else { Target::Next });
        }

        if let Some(captured) = self.cameras.wants_mouse_capture(&self.input) {
            self.set_mouse_captured(captured);
        }

        let supports_wireframe = self.renderer.as_ref().is_some_and(Renderer::supports_wireframe);

        if self
            .keybinds
            .poll_config_changes(&self.input, &mut self.config, supports_wireframe)
        {
            self.apply_config();
        }
    }

    fn reconfigure(&mut self) {
        self.config = self.config_document.config();
        self.apply_config();
    }

    fn apply_config_change(&mut self, key: &str, save: bool) -> anyhow::Result<String> {
        self.reconfigure();
        let mut description = format!("{key} {}", self.config_document.get(key)?);

        if save && ConfigKey::find(key).is_some_and(|key| !key.persisted) {
            description.push_str(" (debug settings are never saved)");
        } else if save {
            self.config_document.save()?;
            description.push_str(&format!(" (saved to {})", self.config_document.path().display()));
        }

        Ok(description)
    }

    fn apply_config(&mut self) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        renderer.configure(&self.config.render, &self.config.debug);
        let skybox = self.world.as_ref().and_then(ClientWorld::skybox);
        renderer.scene_mut().skybox = if self.config.render.stars { skybox } else { None };
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        let Some(renderer) = &mut self.renderer else {
            return Ok(());
        };

        if let Some(world) = &self.world {
            world.sync(renderer);
        }

        renderer.draw(self.cameras.camera())
    }

    fn set_mouse_captured(&mut self, captured: bool) {
        let Some(window) = &self.window else {
            return;
        };

        if captured {
            let grabbed = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));

            if grabbed.is_err() {
                return;
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
        }

        window.set_cursor_visible(!captured);
        self.mouse_captured = captured;
    }

    fn set_camera_mode(&mut self, mode: CameraMode) {
        if self.cameras.set_mode(mode) {
            self.set_mouse_captured(false);
        }
    }

    fn cycle_target(&mut self, target: Target) {
        let command = CameraTargetCommand {
            target,
            distance: None,
            pitch: None,
            yaw: None,
        };

        match command.run(self) {
            Ok(description) => println!("{description}"),
            Err(err) => tracing::warn!("{err:#}"),
        }
    }

    fn window_attributes() -> WindowAttributes {
        let attributes = Window::default_attributes().with_title(APP_ID);
        #[cfg(all(unix, not(target_vendor = "apple")))]
        let attributes = winit::platform::wayland::WindowAttributesExtWayland::with_name(attributes, APP_ID, APP_ID);

        attributes
    }
}

impl CommandExecutor for App {
    fn session(&mut self) -> &mut ServerSession {
        &mut self.session
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }

    fn completion_values(&self, kind: CompletionKind) -> Vec<CompletionCandidate> {
        match kind {
            CompletionKind::Body => self
                .world
                .as_ref()
                .map(ClientWorld::body_candidates)
                .unwrap_or_default(),
            CompletionKind::ConfigKey => ConfigKey::ALL
                .iter()
                .map(|key| CompletionCandidate::new(key.path))
                .collect(),
            CompletionKind::ConfigValue => ConfigKey::ALL
                .iter()
                .flat_map(|key| {
                    key.kind.values().into_iter().map(|value| CompletionCandidate {
                        value: value.to_string(),
                        description: None,
                        scope: Some(key.path.to_string()),
                    })
                })
                .collect(),
        }
    }

    fn run_client(&mut self, command: ClientCommand) -> anyhow::Result<String> {
        match command {
            ClientCommand::CameraMode(command) => command.run(self),
            ClientCommand::CameraTarget(command) => command.run(self),
            ClientCommand::CameraLookAt(command) => command.run(self),
            ClientCommand::ConfigGet(command) => command.run(self),
            ClientCommand::ConfigSet(command) => command.run(self),
            ClientCommand::ConfigToggle(command) => command.run(self),
            ClientCommand::ConfigSave(command) => command.run(self),
            ClientCommand::ConfigReload(command) => command.run(self),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        if let Err(err) = self.initialize(event_loop) {
            self.error = Some(err);
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                let _ = self.session.send(ClientMessage::Leave(Leave));
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::Focused(false) => self.input.clear(),
            WindowEvent::KeyboardInput { event, .. } => self.input.handle_key(&event),
            WindowEvent::MouseInput { state, button, .. } => self.input.handle_mouse_button(button, state),
            WindowEvent::MouseWheel { delta, .. } => self.input.handle_scroll(delta),
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.redraw() {
                    self.error = Some(err);
                    event_loop.exit();
                } else if self.exit_requested {
                    let _ = self.session.send(ClientMessage::Leave(Leave));
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event
            && self.mouse_captured
        {
            self.input.handle_mouse_motion(DVec2::from(delta));
        }
    }
}
