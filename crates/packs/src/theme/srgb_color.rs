use anyhow::{Context, ensure};

const HEX_PREFIX: char = '#';
const HEX_DIGITS: usize = 6;
const HEX_RADIX: u32 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SrgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl SrgbColor {
    pub fn parse(text: &str) -> anyhow::Result<SrgbColor> {
        let digits = text
            .strip_prefix(HEX_PREFIX)
            .with_context(|| format!("{text:?} is not a color like \"#1D7AFC\""))?;
        ensure!(
            digits.len() == HEX_DIGITS && digits.chars().all(|digit| digit.is_ascii_hexdigit()),
            "{text:?} is not a color like \"#1D7AFC\""
        );
        let [_, red, green, blue] = u32::from_str_radix(digits, HEX_RADIX)?.to_be_bytes();

        Ok(SrgbColor { red, green, blue })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors() {
        let color = SrgbColor::parse("#1D7AFC").unwrap();
        assert_eq!(
            color,
            SrgbColor {
                red: 0x1D,
                green: 0x7A,
                blue: 0xFC
            }
        );
    }

    #[test]
    fn rejects_malformed_colors() {
        for text in ["1D7AFC", "#1D7AF", "#1D7AFCFF", "#+D7AFC", "#GGGGGG"] {
            assert!(SrgbColor::parse(text).is_err(), "{text}");
        }
    }
}
