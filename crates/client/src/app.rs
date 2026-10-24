mod bind_actions;
mod body_step;
mod camera_controller;
mod client_command_handler;
mod client_world;
mod command_handlers;
mod command_progress;
mod command_task;
mod forwarded_command;
mod frame_timing;
mod immediate_command_handler;
mod pointer_input;
mod running_task;
mod snapshot;
mod task_status;
mod update_schedule;
mod view_controllers;
mod window_visibility;

use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use anyhow::Context;
use glam::DVec2;
use resource_vulkan::Resources;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use xsa_commands::command::{ClientCommand, LocalCommand, Overlay, ViewControllerId};
use xsa_commands::completion::{CompletionCandidate, CompletionKind};
use xsa_commands::logging::ShellOutputs;
use xsa_commands::router::{CommandExecutor, CommandInvocation, CommandReply, CommandRouter, Output};
use xsa_config::ConfigStore;
use xsa_packs::PackStack;
use xsa_proto::frame::ServerFrame;
use xsa_proto::network::DisconnectReason;
use xsa_proto::session::{ServerSession, SessionState};
use xsa_proto::world::WorldState;
use xui::{
    Alignment, ColorScheme, ColorSchemeKey, Environment, ForegroundStyleKey, Interface, NavigationController, Point,
    ScaleFactorKey, Size, View, ZStack,
};

use crate::BuildIdentity;
use crate::camera::CameraSwitch;
use crate::config::Config;
use crate::input::Input;
use crate::renderer::{FrameTarget, Renderer, ShaderBinaries};
use crate::theme::{Theme, ThemeKey};
use crate::ui::{
    ClockReading, DebugOverlayView, GameCommands, InputDisabledView, SessionEnvironmentKey, ViewControllerFactory,
};
use camera_controller::CameraController;
use client_command_handler::ClientCommandHandler;
pub(crate) use client_world::ClientWorld;
use command_progress::CommandProgress;
use forwarded_command::ForwardedCommand;
use frame_timing::FrameTiming;
use running_task::RunningTask;
use task_status::TaskStatus;
use update_schedule::UpdateSchedule;
use window_visibility::WindowVisibility;

const APP_ID: &str = "xsa";
const BASE_PACK: &str = "base";

// Declaration order is drop order: the world and resources hold GPU objects, so they drop before the renderer.
pub struct App {
    world: Option<ClientWorld>,
    resources: Option<Resources>,
    renderer: Option<Renderer>,
    window: Option<Window>,
    error: Option<anyhow::Error>,
    session: ServerSession,
    router: CommandRouter,
    invocations: UnboundedReceiver<CommandInvocation>,
    shell_outputs: ShellOutputs,
    exit_requested: bool,
    tasks: Vec<RunningTask>,
    forwarded: Vec<ForwardedCommand>,
    packs_directory: PathBuf,
    input: Input,
    cameras: Option<CameraController>,
    config_store: ConfigStore<Config>,
    config: Config,
    mouse_captured: bool,
    cursor: Point,
    pointer_moved: bool,
    schedule: UpdateSchedule,
    interface: Interface,
    theme: Option<Rc<Theme>>,
    frame_timing: FrameTiming,
    navigation: NavigationController,
    view_controllers: ViewControllerFactory,
    commands: GameCommands,
    overlays: HashSet<Overlay>,
    action_replies: Vec<oneshot::Receiver<Output>>,
    build: BuildIdentity,
}

impl App {
    pub fn new(
        session: ServerSession,
        invocations: UnboundedReceiver<CommandInvocation>,
        shell_outputs: ShellOutputs,
        packs_directory: PathBuf,
        config_store: ConfigStore<Config>,
        build: BuildIdentity,
    ) -> Self {
        let commands = GameCommands::default();
        let navigation = NavigationController::new();
        let view_controllers = ViewControllerFactory {
            navigation: navigation.navigation(),
            commands: commands.clone(),
        };

        let mut app = Self {
            world: None,
            resources: None,
            renderer: None,
            window: None,
            error: None,
            session,
            router: CommandRouter::default(),
            invocations,
            shell_outputs,
            exit_requested: false,
            tasks: Vec::new(),
            forwarded: Vec::new(),
            packs_directory,
            input: Input::default(),
            cameras: None,
            config: config_store.config().clone(),
            config_store,
            mouse_captured: false,
            cursor: Point::default(),
            pointer_moved: false,
            schedule: UpdateSchedule::new(),
            interface: Interface::new(),
            theme: None,
            frame_timing: FrameTiming::new(),
            navigation,
            view_controllers,
            commands,
            overlays: HashSet::new(),
            action_replies: Vec::new(),
            build,
        };
        app.set_view_controller(ViewControllerId::MainMenu);

        app
    }

    pub fn into_result(self) -> anyhow::Result<()> {
        self.error.map_or(Ok(()), Err)
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = self.window.insert(event_loop.create_window(App::window_attributes())?);
        let base = PackStack::load(&self.packs_directory, &[BASE_PACK.to_string()])?;
        let shaders = ShaderBinaries::load(&base)?;
        let renderer = Renderer::new(window, &shaders, &self.config)?;
        self.resources = Some(renderer.create_resources());
        self.renderer = Some(renderer);
        self.theme = Some(Rc::new(Theme::load(&base, self.interface.fonts_mut())?));
        self.cameras = Some(CameraController::new(window.inner_size(), &self.config.debug.camera));
        window.request_redraw();
        self.apply_config();

        Ok(())
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let gpu_milliseconds = self
            .renderer
            .as_ref()
            .and_then(|renderer| renderer.statistics().gpu_milliseconds);

        if let Some(delta_seconds) = self.schedule.record_frame(now) {
            self.frame_timing.record(delta_seconds, gpu_milliseconds);
        }

        self.update(now)?;
        self.draw_window()?;
        self.finish_update();

        if let Some(window) = &self.window {
            window.request_redraw();
        }

        Ok(())
    }

    fn update_without_window(&mut self, now: Instant) -> anyhow::Result<()> {
        self.update(now)?;

        if self.tasks.iter().any(|running| running.task.needs_frames()) {
            self.draw_offscreen()?;
        }

        self.finish_update();

        Ok(())
    }

    fn update(&mut self, now: Instant) -> anyhow::Result<()> {
        let delta_seconds = self.schedule.record_update(now);
        self.offer_pointer_move();
        let environment = self.environment();
        self.navigation.advance(delta_seconds as f32, &environment);
        self.poll_connection()?;

        if let Some(resources) = &self.resources {
            resources.update()?;
        }

        self.advance_loading();
        self.poll_invocations();
        self.poll_input();
        self.run_game_commands();
        let idle = Input::default();
        let mut scrolled_speed = None;
        let input = if self.is_game_active() && self.config.input.enabled {
            &self.input
        } else {
            &idle
        };

        if let Some(world) = &mut self.world {
            if let Some(state) = self.session.state() {
                world.advance(state.time());
            }

            if let Some(cameras) = &mut self.cameras {
                scrolled_speed = cameras.update(world, input, delta_seconds);
            }
        }

        self.follow_camera_switch();

        if let Some(meters_per_second) = scrolled_speed {
            let speed = meters_per_second as f32;
            let result = self.config_store.update(|config| config.debug.camera.speed = speed);
            self.reconfigure();

            if let Err(err) = result {
                tracing::warn!("{err:#}");
            }
        }

        Ok(())
    }

    fn finish_update(&mut self) {
        self.poll_tasks();
        self.input.end_frame();
    }

    fn conclude(&mut self, event_loop: &ActiveEventLoop, result: anyhow::Result<()>) {
        if let Err(err) = result {
            self.error = Some(err);
            self.shut_down(event_loop);
        } else if self.exit_requested {
            self.shut_down(event_loop);
        }
    }

    fn poll_connection(&mut self) -> anyhow::Result<()> {
        while let Some(frame) = self.session.poll()? {
            self.router.handle_frame(&frame, &self.session);

            match frame {
                ServerFrame::Snapshot(state) => self
                    .join(&state)
                    .with_context(|| format!("joining simulation {}", state.simulation))?,
                ServerFrame::Event(_) | ServerFrame::Reply { .. } => {}
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

        for forwarded in std::mem::take(&mut self.forwarded) {
            // Nothing waits for the reply of a command a bind ran; it is dropped.
            let reply = forwarded
                .reply
                .unwrap_or_else(|| CommandReply::new(oneshot::channel().0));
            router.forward(forwarded.command, reply, self);
        }

        self.router = router;
    }

    fn join(&mut self, state: &WorldState) -> anyhow::Result<()> {
        let renderer = self.renderer.as_mut().context("the renderer is not ready")?;
        let resources = self.resources.as_ref().context("resources are not ready")?;
        let world = ClientWorld::load(&self.packs_directory, state, renderer, resources)?;
        let cameras = self.cameras.as_mut().context("the cameras are not ready")?;
        cameras.focus(&world);
        self.world = Some(world);
        self.apply_config();

        Ok(())
    }

    fn poll_input(&mut self) {
        for action in self.config.input.bind.triggered(&self.input) {
            self.run_bind_action(action);
        }

        if !self.is_game_active() || !self.config.input.enabled {
            if self.mouse_captured {
                self.set_mouse_captured(false);
            }

            return;
        }

        if let Some(captured) = self
            .cameras
            .as_ref()
            .and_then(|cameras| cameras.wants_mouse_capture(&self.input))
        {
            self.set_mouse_captured(captured);
        }
    }

    fn reconfigure(&mut self) {
        self.config = self.config_store.config().clone();
        self.apply_config();
    }

    fn apply_config(&mut self) {
        self.shell_outputs.apply(&self.config.logging);

        for chord in self.config.input.bind.shared_chords() {
            tracing::warn!("{} is bound to more than one action", chord.name());
        }

        self.navigation
            .set_duration_scale(self.config.animation.ui_duration_scale());

        if let Some(cameras) = &mut self.cameras {
            cameras.set_animation_duration_scale(f64::from(self.config.animation.camera_duration_scale()));
            cameras.set_debug_camera_config(&self.config.debug.camera);
        }

        if let Some(resources) = &self.resources {
            resources.set_limits(self.config.service.resources.limits());
        }

        let Some(renderer) = &mut self.renderer else {
            return;
        };

        renderer.configure(&self.config);
        let skybox = self.world.as_ref().and_then(ClientWorld::skybox);
        renderer.scene_mut().skybox = if self.config.video.stars { skybox } else { None };
    }

    fn draw_window(&mut self) -> anyhow::Result<()> {
        self.prepare_frame()?;
        let (Some(renderer), Some(cameras), Some(window)) = (&mut self.renderer, &self.cameras, &self.window) else {
            return Ok(());
        };

        renderer.draw(cameras.camera(), FrameTarget::Window { window })
    }

    fn draw_offscreen(&mut self) -> anyhow::Result<()> {
        self.prepare_frame()?;
        let (Some(renderer), Some(cameras)) = (&mut self.renderer, &self.cameras) else {
            return Ok(());
        };

        renderer.draw(cameras.camera(), FrameTarget::Offscreen)
    }

    fn prepare_frame(&mut self) -> anyhow::Result<()> {
        self.update_user_interface()?;
        let (Some(renderer), Some(cameras), Some(world)) = (&mut self.renderer, &self.cameras, &self.world) else {
            return Ok(());
        };

        world.sync(renderer);
        renderer.set_illuminance(world.illuminance_at(cameras.camera().position) as f32);

        Ok(())
    }

    // Keys whose source does not exist yet, such as the theme before the window opens, keep their default.
    fn environment(&self) -> Environment {
        let color_scheme = ColorScheme::Dark;
        let mut environment = Environment::default()
            .with::<ColorSchemeKey>(color_scheme)
            .with::<SessionEnvironmentKey>(self.session.state().map(SessionState::environment));

        if let Some(window) = &self.window {
            environment = environment.with::<ScaleFactorKey>(window.scale_factor() as f32);
        }

        if let Some(theme) = &self.theme {
            environment = environment
                .with::<ForegroundStyleKey>(theme.color(&theme.foreground().default, color_scheme))
                .with::<ThemeKey>(Some(theme.clone()));
        }

        environment
    }

    fn update_user_interface(&mut self) -> anyhow::Result<()> {
        let environment = self.environment();
        let (Some(renderer), Some(window), Some(_)) = (&mut self.renderer, &self.window, &self.theme) else {
            return Ok(());
        };
        let window_size = window.inner_size();
        let viewport = Size {
            width: window_size.width as f32,
            height: window_size.height as f32,
        };
        let fill = Some(f32::INFINITY);
        let position = self
            .world
            .as_ref()
            .zip(self.cameras.as_ref())
            .and_then(|(world, cameras)| {
                let camera = cameras.camera();
                cameras
                    .anchor()
                    .map(|anchor| world.position_reading(anchor.body(), camera.position, camera.orientation))
            });
        let debug_overlay = self.overlays.contains(&Overlay::DebugOverlay).then(|| {
            DebugOverlayView {
                frames_per_second: self.frame_timing.frames_per_second(),
                gpu_milliseconds: self.frame_timing.gpu_milliseconds(),
                build: self.build.clone(),
                statistics: renderer.statistics(),
                camera: self.cameras.as_ref().and_then(CameraController::reading),
                position,
                clock: self.session.state().map(|state| ClockReading {
                    time: state.time(),
                    time_flow: state.time_flow(),
                    rate: state.rate(),
                }),
            }
            .max_frame(fill, fill, Alignment::TOP_LEADING)
        });
        let input_disabled =
            (!self.config.input.enabled).then(|| InputDisabledView.max_frame(fill, fill, Alignment::TOP));
        let root = ZStack::new((self.navigation.view(), debug_overlay, input_disabled));
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

    fn follow_camera_switch(&mut self) {
        let Some(switch) = self.cameras.as_mut().and_then(CameraController::take_switch) else {
            return;
        };

        self.set_mouse_captured(false);

        if switch == CameraSwitch::Cut
            && let Some(renderer) = &mut self.renderer
        {
            renderer.reset_history();
        }
    }

    fn follow(&mut self, progress: CommandProgress, reply: Option<CommandReply>) {
        match progress {
            CommandProgress::Done(result) => App::finish(result, reply),
            CommandProgress::Running(task) => self.tasks.push(RunningTask { task, reply }),
            CommandProgress::Forward(command) => self.forwarded.push(ForwardedCommand { command, reply }),
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

        self.session.close(DisconnectReason::Left);
        event_loop.exit();
    }

    fn set_window_visibility(&mut self, occluded: bool) {
        let visibility = if occluded {
            WindowVisibility::Occluded
        } else {
            WindowVisibility::Visible
        };
        self.schedule.set_visibility(visibility);

        if let (WindowVisibility::Visible, Some(window)) = (visibility, &self.window) {
            window.request_redraw();
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
            CompletionKind::ConfigKey | CompletionKind::ConfigValue => {
                CompletionCandidate::config(kind, self.config_store.keys())
            }
        }
    }

    fn run_local(&mut self, command: LocalCommand) -> Output {
        let result = command.run(&mut self.config_store);
        self.reconfigure();

        match result {
            Ok(text) => Output::success(text),
            Err(err) => Output::failure(format!("{err:#}")),
        }
    }

    fn run_client(&mut self, command: ClientCommand, reply: CommandReply) {
        let progress = match command {
            ClientCommand::CameraTarget(command) => command.start(self),
            ClientCommand::CameraTeleport(command) => command.start(self),
            ClientCommand::CameraLookAt(command) => command.start(self),
            ClientCommand::CameraSnap(command) => command.start(self),
            ClientCommand::InterfacePush(command) => command.start(self),
            ClientCommand::InterfacePop(command) => command.start(self),
            ClientCommand::InterfaceSet(command) => command.start(self),
            ClientCommand::InterfaceShow(command) => command.start(self),
            ClientCommand::InterfaceHide(command) => command.start(self),
            ClientCommand::InterfaceToggle(command) => command.start(self),
            ClientCommand::TimeWarp(command) => command.start(self),
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

                if let Some(cameras) = &mut self.cameras {
                    cameras.resize(size);
                }
            }
            WindowEvent::Focused(false) => self.input.clear(),
            WindowEvent::KeyboardInput { event, .. } => self.input.handle_key(&event),
            WindowEvent::MouseInput { state, button, .. } => self.handle_mouse_button(button, state),
            WindowEvent::CursorMoved { position, .. } => self.handle_cursor_moved(position),
            WindowEvent::CursorLeft { .. } => self.handle_cursor_left(),
            WindowEvent::MouseWheel { delta, .. } => self.input.handle_scroll(delta),
            WindowEvent::Occluded(occluded) => self.set_window_visibility(occluded),
            WindowEvent::RedrawRequested if self.schedule.visibility() == WindowVisibility::Visible => {
                let result = self.redraw();
                self.conclude(event_loop, result);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() || event_loop.exiting() {
            return;
        }

        let now = Instant::now();

        if self.schedule.is_due(now) {
            let result = self.update_without_window(now);
            self.conclude(event_loop, result);
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.schedule.next_wake(Instant::now())));
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event
            && self.mouse_captured
        {
            self.input.handle_mouse_motion(DVec2::from(delta));
        }
    }
}
