use std::fmt;

use anstyle::{Effects, RgbColor, Style};

#[derive(Clone, Copy)]
pub(super) struct SequenceNumber {
    value: u64,
}

impl SequenceNumber {
    // Stepping the hue by the golden ratio keeps consecutive numbers far apart on the color wheel.
    const HUE_STEP: f64 = 0.618_033_988_749_895;
    const SATURATION: f64 = 0.7;
    const LIGHTNESS: f64 = 0.65;

    pub fn new(value: u64) -> SequenceNumber {
        SequenceNumber { value }
    }

    fn style(self) -> Style {
        let hue = (self.value as f64 * SequenceNumber::HUE_STEP).fract();

        hsl_color(hue, SequenceNumber::SATURATION, SequenceNumber::LIGHTNESS)
            .on_default()
            .effects(Effects::BOLD)
    }
}

impl fmt::Display for SequenceNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = self.style();

        write!(formatter, "ipc {style}#{}{style:#}", self.value)
    }
}

// Hue in turns, saturation and lightness in [0, 1].
fn hsl_color(hue: f64, saturation: f64, lightness: f64) -> RgbColor {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = hue * 6.0;
    let second = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let (red, green, blue) = match sector as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let lift = lightness - chroma / 2.0;
    let channel = |value: f64| ((value + lift) * f64::from(u8::MAX)).round() as u8;

    RgbColor(channel(red), channel(green), channel(blue))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_hues_convert_to_primary_colors() {
        assert_eq!(hsl_color(0.0, 1.0, 0.5), RgbColor(255, 0, 0));
        assert_eq!(hsl_color(1.0 / 3.0, 1.0, 0.5), RgbColor(0, 255, 0));
        assert_eq!(hsl_color(2.0 / 3.0, 1.0, 0.5), RgbColor(0, 0, 255));
    }

    #[test]
    fn zero_saturation_is_grey() {
        assert_eq!(hsl_color(0.4, 0.0, 0.5), RgbColor(128, 128, 128));
    }
}
