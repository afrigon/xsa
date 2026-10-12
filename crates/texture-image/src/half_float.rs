const HALF_EXPONENT_BIAS: i32 = 15;
const HALF_MANTISSA_BITS: u32 = 10;
const HALF_EXPONENT_MASK: u16 = 0x1f;
const HALF_MANTISSA_MASK: u16 = 0x3ff;
const HALF_SIGN_SHIFT: u32 = 15;
// The exponent that marks infinity and NaN.
const HALF_SPECIAL_EXPONENT: i32 = 0x1f;

// IEEE 754 binary16 to binary32.
pub(crate) fn half_to_f32(bits: u16) -> f32 {
    let sign = if bits >> HALF_SIGN_SHIFT == 1 { -1.0 } else { 1.0 };
    let exponent = ((bits >> HALF_MANTISSA_BITS) & HALF_EXPONENT_MASK) as i32;
    let mantissa = f32::from(bits & HALF_MANTISSA_MASK) / (1 << HALF_MANTISSA_BITS) as f32;

    match exponent {
        0 => sign * mantissa * 2.0_f32.powi(1 - HALF_EXPONENT_BIAS),
        HALF_SPECIAL_EXPONENT if mantissa == 0.0 => sign * f32::INFINITY,
        HALF_SPECIAL_EXPONENT => f32::NAN,
        _ => sign * (1.0 + mantissa) * 2.0_f32.powi(exponent - HALF_EXPONENT_BIAS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_floats_decode_to_their_values() {
        assert_eq!(half_to_f32(0x3c00), 1.0);
        assert_eq!(half_to_f32(0xc000), -2.0);
        assert_eq!(half_to_f32(0x3555), 0.333_251_95);
        assert_eq!(half_to_f32(0x0001), 2.0_f32.powi(-24));
        assert_eq!(half_to_f32(0x7c00), f32::INFINITY);
    }
}
