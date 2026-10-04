mod bind_actions;
mod body_step;
mod camera_controller;
mod client_command_handler;
mod client_world;
mod command_handlers;
mod command_progress;
mod command_task;
mod frame_rate;
mod immediate_command_handler;
mod pointer_input;
mod running_task;
mod snapshot;
mod task_status;

use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use anyhow::{Context, bail};
use glam::DVec2;
use tokio::sync::mpsc::UnboundedReceiver;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use xsa_commands::command::ClientCommand;
use xsa_commands::completion::{CompletionCandidate, CompletionKind};
use xsa_commands::router::{CommandExecutor, CommandInvocation, CommandReply, CommandRouter};
use xsa_packs::PackStack;
use xsa_proto::event::{ServerEvent, WorldState};
use xsa_proto::message::{ClientMessage, Leave};
use xsa_proto::session::ServerSession;
use xui::{
    Alignment, ColorScheme, ColorSchemeKey, Environment, ForegroundStyleKey, Interface, Point, ScaleFactorKey, Size,
    View, ZStack,
};

use crate::camera::CameraMode;
use crate::config::Config;
use crate::document::{ConfigDocument, ConfigKey};
use crate::input::Input;
use crate::renderer::{CapturedImage, Renderer, ShaderBinaries};
use crate::theme::{Theme, ThemeKey};
use crate::ui::{DebugOverlayView, HudTargetView};
use camera_controller::CameraController;
use client_command_handler::ClientCommandHandler;
use client_world::ClientWorld;
use command_progress::CommandProgress;
use frame_rate::FrameRate;
use running_task::RunningTask;
use task_status::TaskStatus;

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
    tasks: Vec<RunningTask>,
    packs_directory: PathBuf,
    world: Option<ClientWorld>,
    input: Input,
    cameras: CameraController,
    config_document: ConfigDocument,
    config: Config,
    mouse_captured: bool,
    cursor: Point,
    pointer_moved: bool,
    last_frame: Option<Instant>,
    interface: Interface,
    theme: Option<Rc<Theme>>,
    frame_rate: FrameRate,
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
            tasks: Vec::new(),
            packs_directory,
            world: None,
            input: Input::default(),
            cameras: CameraController::new(),
            config: config_document.config(),
            config_document,
            mouse_captured: false,
            cursor: Point::default(),
            pointer_moved: false,
            last_frame: None,
            interface: Interface::new(),
            theme: None,
            frame_rate: FrameRate::new(),
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
        self.theme = Some(Rc::new(Theme::load(&base, self.interface.fonts_mut())?));
        window.request_redraw();
        self.window = Some(window);

        Ok(())
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let delta_seconds = self.last_frame.map_or(0.0, |last| (now - last).as_secs_f64());
        self.last_frame = Some(now);
        self.frame_rate.record(delta_seconds);
        self.offer_pointer_move();

        self.update(delta_seconds)?;
        self.draw()?;
        self.poll_tasks();
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
        for action in self.config.bind.triggered(&self.input) {
            self.run_bind_action(action);
        }

        if let Some(captured) = self.cameras.wants_mouse_capture(&self.input) {
            self.set_mouse_captured(captured);
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

    fn capture(&mut self) -> anyhow::Result<CapturedImage> {
        self.update_user_interface()?;
        let renderer = self.renderer.as_mut().context("the renderer is not ready")?;

        if let Some(world) = &self.world {
            world.sync(renderer);
        }

        renderer.capture(self.cameras.camera())
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.update_user_interface()?;
        let Some(renderer) = &mut self.renderer else {
            return Ok(());
        };

        if let Some(world) = &self.world {
            world.sync(renderer);
        }

        renderer.draw(self.cameras.camera())
    }

    fn update_user_interface(&mut self) -> anyhow::Result<()> {
        let (Some(renderer), Some(window), Some(theme)) = (&mut self.renderer, &self.window, &self.theme) else {
            return Ok(());
        };
        let color_scheme = ColorScheme::Dark;
        let environment = Environment::default()
            .with::<ScaleFactorKey>(window.scale_factor() as f32)
            .with::<ColorSchemeKey>(color_scheme)
            .with::<ForegroundStyleKey>(theme.color(&theme.foreground().default, color_scheme))
            .with::<ThemeKey>(Some(theme.clone()));
        let window_size = window.inner_size();
        let viewport = Size {
            width: window_size.width as f32,
            height: window_size.height as f32,
        };
        let target = self
            .world
            .as_ref()
            .zip(self.cameras.target_body())
            .map(|(world, body)| world.body(body).id.to_string());
        let fill = Some(f32::INFINITY);
        let root = ZStack::new((
            target.map(|name| HudTargetView { name }.max_frame(fill, fill, Alignment::TOP)),
            DebugOverlayView {
                frames_per_second: self.frame_rate.frames_per_second(),
                triangles: renderer.statistics().triangles,
            }
            .max_frame(fill, fill, Alignment::TOP_LEADING),
        ));
        let draw_list = self.interface.render(&root, viewport, &environment)?;
        renderer.set_user_interface(draw_list);

        Ok(())
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

    fn follow(&mut self, progress: CommandProgress, reply: Option<CommandReply>) {
        match progress {
            CommandProgress::Done(result) => App::finish(result, reply),
            CommandProgress::Running(task) => self.tasks.push(RunningTask { task, reply }),
        }
    }

    fn finish(result: anyhow::Result<String>, reply: Option<CommandReply>) {
        if let Some(reply) = reply {
            reply.send(result);

            return;
        }

        match result {
            Ok(description) => tracing::info!("{description}"),
            Err(err) => tracing::warn!("{err:#}"),
        }
    }

    fn poll_tasks(&mut self) {
        let mut waiting = Vec::new();

        for mut running in std::mem::take(&mut self.tasks) {
            match running.task.poll(self) {
                TaskStatus::Pending => waiting.push(running),
                TaskStatus::Done(result) => App::finish(result, running.reply),
            }
        }

        waiting.append(&mut self.tasks);
        self.tasks = waiting;
    }

    fn shut_down(&mut self, event_loop: &ActiveEventLoop) {
        for running in std::mem::take(&mut self.tasks) {
            App::finish(Err(anyhow::anyhow!("the game is exiting")), running.reply);
        }

        let _ = self.session.send(ClientMessage::Leave(Leave));
        event_loop.exit();
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
            CompletionKind::ConfigKey => ConfigKey::all()
                .into_iter()
                .map(|key| CompletionCandidate::new(key.path))
                .collect(),
            CompletionKind::ConfigValue => ConfigKey::all()
                .into_iter()
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

    fn run_client(&mut self, command: ClientCommand, reply: CommandReply) {
        let progress = match command {
            ClientCommand::CameraMode(command) => command.start(self),
            ClientCommand::CameraTarget(command) => command.start(self),
            ClientCommand::CameraLookAt(command) => command.start(self),
            ClientCommand::CameraSnap(command) => command.start(self),
            ClientCommand::ConfigGet(command) => command.start(self),
            ClientCommand::ConfigSet(command) => command.start(self),
            ClientCommand::ConfigToggle(command) => command.start(self),
            ClientCommand::ConfigSave(command) => command.start(self),
            ClientCommand::ConfigReload(command) => command.start(self),
        };

        self.follow(progress, Some(reply));
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
            WindowEvent::CloseRequested => self.shut_down(event_loop),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::Focused(false) => self.input.clear(),
            WindowEvent::KeyboardInput { event, .. } => self.input.handle_key(&event),
            WindowEvent::MouseInput { state, button, .. } => self.handle_mouse_button(button, state),
            WindowEvent::CursorMoved { position, .. } => self.handle_cursor_moved(position),
            WindowEvent::CursorLeft { .. } => self.handle_cursor_left(),
            WindowEvent::MouseWheel { delta, .. } => self.input.handle_scroll(delta),
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.redraw() {
                    self.error = Some(err);
                    self.shut_down(event_loop);
                } else if self.exit_requested {
                    self.shut_down(event_loop);
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
