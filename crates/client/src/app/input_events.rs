use anyhow::{Context, ensure};
use glam::DVec2;
use xsa_input::{InputButton, InputEvent, InputEventKind, InputSource, ScrollDelta};

use super::App;

impl App {
    pub(super) fn handle_input(&mut self, event: InputEvent) {
        if !self.config.input.accepts(&event) {
            return;
        }

        match event.kind {
            InputEventKind::Pressed {
                button: InputButton::Mouse(button),
            } => self.handle_mouse_pressed(button),
            InputEventKind::Released {
                button: InputButton::Mouse(button),
            } => self.handle_mouse_released(button),
            InputEventKind::Pressed { button } => self.input.press(button),
            InputEventKind::Released { button } => self.input.release(button),
            InputEventKind::CursorMoved { position } => self.handle_cursor_moved(position),
            InputEventKind::CursorLeft => self.handle_cursor_left(),
            InputEventKind::Moved { delta } => self.input.handle_mouse_motion(delta),
            InputEventKind::Scrolled { delta } => self.input.handle_scroll(delta),
        }
    }

    pub(super) fn inject(&mut self, kinds: impl IntoIterator<Item = InputEventKind>) -> anyhow::Result<()> {
        ensure!(self.window.is_some(), "the window is not open");
        ensure!(self.config.input.enabled, "input is disabled (input.enabled)");
        ensure!(
            self.config.input.commands.enabled,
            "command input is disabled (input.commands.enabled)"
        );

        for kind in kinds {
            self.handle_input(InputEvent {
                source: InputSource::Command,
                kind,
            });
        }

        Ok(())
    }

    pub(super) fn inject_cursor_position(&mut self, logical: DVec2) -> anyhow::Result<()> {
        ensure!(
            !self.mouse_captured,
            "the mouse is captured and has no cursor; move it with input cursor move"
        );
        let position = logical * self.scale_factor()?;

        self.inject([InputEventKind::CursorMoved { position }])
    }

    // Mouse look reads device counts, not pixels: one logical pixel of offset is one count.
    pub(super) fn inject_mouse_offset(&mut self, logical: DVec2) -> anyhow::Result<()> {
        let kind = if self.mouse_captured {
            InputEventKind::Moved { delta: logical }
        } else {
            let cursor = DVec2::new(f64::from(self.cursor.x), f64::from(self.cursor.y));

            InputEventKind::CursorMoved {
                position: cursor + logical * self.scale_factor()?,
            }
        };

        self.inject([kind])
    }

    pub(super) fn inject_scroll(&mut self, amount: ScrollDelta) -> anyhow::Result<()> {
        let delta = match amount {
            ScrollDelta::Notches { .. } => amount,
            ScrollDelta::Pixels { vertical } => ScrollDelta::Pixels {
                vertical: vertical * self.scale_factor()?,
            },
        };

        self.inject([InputEventKind::Scrolled { delta }])
    }

    fn scale_factor(&self) -> anyhow::Result<f64> {
        Ok(self.window.as_ref().context("the window is not open")?.scale_factor())
    }
}
