const SRGB_MAXIMUM: f32 = 255.0;
const SRGB_LINEAR_THRESHOLD: f32 = 0.04045;
const SRGB_LINEAR_SLOPE: f32 = 12.92;
const SRGB_CURVE_OFFSET: f32 = 0.055;
const SRGB_CURVE_SCALE: f32 = 1.055;
const SRGB_CURVE_EXPONENT: f32 = 2.4;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LinearColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl LinearColor {
    pub const WHITE: LinearColor = LinearColor {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 1.0,
    };

    pub fn from_srgb(red: u8, green: u8, blue: u8) -> LinearColor {
        LinearColor {
            red: LinearColor::decode(red),
            green: LinearColor::decode(green),
            blue: LinearColor::decode(blue),
            alpha: 1.0,
        }
    }

    pub fn opacity(self, opacity: f32) -> LinearColor {
        LinearColor {
            alpha: self.alpha * opacity,
            ..self
        }
    }

    fn decode(channel: u8) -> f32 {
        let encoded = f32::from(channel) / SRGB_MAXIMUM;

        if encoded <= SRGB_LINEAR_THRESHOLD {
            encoded / SRGB_LINEAR_SLOPE
        } else {
            ((encoded + SRGB_CURVE_OFFSET) / SRGB_CURVE_SCALE).powf(SRGB_CURVE_EXPONENT)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_srgb() {
        assert_eq!(LinearColor::from_srgb(0, 0, 0).red, 0.0);
        assert_eq!(LinearColor::from_srgb(255, 255, 255), LinearColor::WHITE);
        assert!((LinearColor::from_srgb(128, 0, 0).red - 0.2158605).abs() < 1e-6);
    }
}
