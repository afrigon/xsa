const HALF_EXPONENT_BIAS: i32 = 15;
const HALF_MANTISSA_BITS: u32 = 10;
const HALF_EXPONENT_MASK: u16 = 0x1f;
const HALF_MANTISSA_MASK: u16 = 0x3ff;
const HALF_SIGN_SHIFT: u32 = 15;
const HALF_SIGN_BIT: u16 = 0x8000;
const HALF_INFINITY: u16 = 0x7c00;
const HALF_QUIET_NAN: u16 = 0x7e00;
// The exponent that marks infinity and NaN.
const HALF_SPECIAL_EXPONENT: i32 = 0x1f;
const SINGLE_EXPONENT_BIAS: i32 = 127;
const SINGLE_MANTISSA_BITS: u32 = 23;
const SINGLE_EXPONENT_MASK: u32 = 0xff;
const SINGLE_MANTISSA_MASK: u32 = 0x7f_ffff;
const SINGLE_IMPLICIT_BIT: u32 = 1 << SINGLE_MANTISSA_BITS;
const SINGLE_SIGN_SHIFT: u32 = 31;
const SINGLE_SPECIAL_EXPONENT: i32 = 0xff;
// The mantissa bits a single-precision float has beyond a half's.
const DROPPED_MANTISSA_BITS: u32 = SINGLE_MANTISSA_BITS - HALF_MANTISSA_BITS;

/// An IEEE 754 binary16 float, as its bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HalfFloat(pub u16);

impl HalfFloat {
    /// The nearest half to a value, ties to even; values beyond its range become infinite.
    pub fn from_f32(value: f32) -> HalfFloat {
        let bits = value.to_bits();
        let sign = if bits >> SINGLE_SIGN_SHIFT == 1 {
            HALF_SIGN_BIT
        } else {
            0
        };
        let exponent = ((bits >> SINGLE_MANTISSA_BITS) & SINGLE_EXPONENT_MASK) as i32;
        let mantissa = bits & SINGLE_MANTISSA_MASK;

        if exponent == SINGLE_SPECIAL_EXPONENT {
            return HalfFloat(sign | if mantissa == 0 { HALF_INFINITY } else { HALF_QUIET_NAN });
        }

        let half_exponent = exponent - SINGLE_EXPONENT_BIAS + HALF_EXPONENT_BIAS;

        if half_exponent >= HALF_SPECIAL_EXPONENT {
            return HalfFloat(sign | HALF_INFINITY);
        }

        if half_exponent > 0 {
            let truncated = (half_exponent as u32) << HALF_MANTISSA_BITS | mantissa >> DROPPED_MANTISSA_BITS;

            return HalfFloat(sign | HalfFloat::rounded(truncated, mantissa, DROPPED_MANTISSA_BITS) as u16);
        }

        // A subnormal half keeps the implicit leading bit in its mantissa, one more bit dropped per exponent below 1.
        let shift = (DROPPED_MANTISSA_BITS + 1) as i32 - half_exponent;

        if shift > (SINGLE_MANTISSA_BITS + 1) as i32 {
            return HalfFloat(sign);
        }

        let shift = shift as u32;

        let mantissa = mantissa | SINGLE_IMPLICIT_BIT;

        HalfFloat(sign | HalfFloat::rounded(mantissa >> shift, mantissa, shift) as u16)
    }

    pub fn to_f32(self) -> f32 {
        let bits = self.0;
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

    // Rounds a value truncated by dropping the low `dropped` bits of `source` to the nearest, ties to even; a carry
    // into the exponent is the correctly rounded result.
    fn rounded(truncated: u32, source: u32, dropped: u32) -> u32 {
        let remainder = source & ((1 << dropped) - 1);
        let halfway = 1 << (dropped - 1);
        let round_up = remainder > halfway || (remainder == halfway && truncated & 1 == 1);

        truncated + u32::from(round_up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_floats_decode_to_their_values() {
        assert_eq!(HalfFloat(0x3c00).to_f32(), 1.0);
        assert_eq!(HalfFloat(0xc000).to_f32(), -2.0);
        assert_eq!(HalfFloat(0x3555).to_f32(), 0.333_251_95);
        assert_eq!(HalfFloat(0x0001).to_f32(), 2.0_f32.powi(-24));
        assert_eq!(HalfFloat(0x7c00).to_f32(), f32::INFINITY);
    }

    #[test]
    fn every_finite_half_encodes_back_to_itself() {
        for bits in 0..=u16::MAX {
            let half = HalfFloat(bits);

            if half.to_f32().is_finite() {
                assert_eq!(HalfFloat::from_f32(half.to_f32()), half, "{bits:#06x}");
            }
        }
    }

    #[test]
    fn floats_round_to_the_nearest_half_ties_to_even() {
        let one_and_half_a_step = 1.0 + 2.0_f32.powi(-11);

        assert_eq!(HalfFloat::from_f32(one_and_half_a_step), HalfFloat(0x3c00));
        assert_eq!(HalfFloat::from_f32(1.0 + 3.0 * 2.0_f32.powi(-11)), HalfFloat(0x3c02));
        assert_eq!(HalfFloat::from_f32(0.333_333), HalfFloat(0x3555));
        assert_eq!(HalfFloat::from_f32(2.0_f32.powi(-25) * 1.5), HalfFloat(0x0001));
        assert_eq!(HalfFloat::from_f32(2.0_f32.powi(-26)), HalfFloat(0x0000));
        assert_eq!(HalfFloat::from_f32(65520.0), HalfFloat(HALF_INFINITY));
        assert_eq!(HalfFloat::from_f32(-0.0), HalfFloat(HALF_SIGN_BIT));
        assert!(HalfFloat::from_f32(f32::NAN).to_f32().is_nan());
    }
}
