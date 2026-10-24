use std::str::FromStr;

use anyhow::{Context, ensure};

const PIXELS_SUFFIX: &str = "px";

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

// A bare number is notches: `3`, `-1.5`, `120px`.
impl FromStr for ScrollDelta {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> anyhow::Result<ScrollDelta> {
        let invalid = || format!("{text:?} is not a scroll amount like 3 (notches) or 120px (pixels)");
        let number = text.strip_suffix(PIXELS_SUFFIX);
        let vertical: f64 = number.unwrap_or(text).parse().ok().with_context(invalid)?;
        ensure!(vertical.is_finite(), invalid());

        Ok(match number {
            Some(_) => ScrollDelta::Pixels { vertical },
            None => ScrollDelta::Notches { vertical },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_number_is_notches_and_px_is_pixels() {
        assert_eq!(
            "3".parse::<ScrollDelta>().unwrap(),
            ScrollDelta::Notches { vertical: 3.0 }
        );
        assert_eq!(
            "-120px".parse::<ScrollDelta>().unwrap(),
            ScrollDelta::Pixels { vertical: -120.0 }
        );
        assert!("3 notches".parse::<ScrollDelta>().is_err());
        assert!("infpx".parse::<ScrollDelta>().is_err());
    }
}
