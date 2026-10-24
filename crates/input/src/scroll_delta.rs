// Positive scrolls away from the user. A wheel turns in notches; a touchpad slides in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDelta {
    Notches { vertical: f64 },
    Pixels { vertical: f64 },
}

#[cfg(feature = "winit")]
impl From<winit::event::MouseScrollDelta> for ScrollDelta {
    fn from(delta: winit::event::MouseScrollDelta) -> ScrollDelta {
        match delta {
            winit::event::MouseScrollDelta::LineDelta(_, vertical) => ScrollDelta::Notches {
                vertical: f64::from(vertical),
            },
            winit::event::MouseScrollDelta::PixelDelta(position) => ScrollDelta::Pixels { vertical: position.y },
        }
    }
}
