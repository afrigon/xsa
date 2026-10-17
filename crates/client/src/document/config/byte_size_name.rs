use crate::config::ByteSize;

struct ByteUnit {
    name: &'static str,
    bytes: u64,
}

const KIBI: u64 = 1024;
const KILO: u64 = 1000;

// Binary units first, largest first, so a size is written in the largest unit that divides it.
const UNITS: [ByteUnit; 7] = [
    ByteUnit {
        name: "GiB",
        bytes: KIBI * KIBI * KIBI,
    },
    ByteUnit {
        name: "MiB",
        bytes: KIBI * KIBI,
    },
    ByteUnit {
        name: "KiB",
        bytes: KIBI,
    },
    ByteUnit {
        name: "GB",
        bytes: KILO * KILO * KILO,
    },
    ByteUnit {
        name: "MB",
        bytes: KILO * KILO,
    },
    ByteUnit {
        name: "KB",
        bytes: KILO,
    },
    ByteUnit { name: "B", bytes: 1 },
];

impl ByteSize {
    // A whole number and a unit, such as `512MiB` or `2GB`.
    pub fn from_config_name(name: &str) -> Option<ByteSize> {
        UNITS.iter().find_map(|unit| {
            let count = name.strip_suffix(unit.name)?.parse::<u64>().ok()?;

            Some(ByteSize {
                bytes: count.checked_mul(unit.bytes)?,
            })
        })
    }

    pub fn config_name(self) -> String {
        let unit = UNITS
            .iter()
            .find(|unit| self.bytes.is_multiple_of(unit.bytes) && self.bytes >= unit.bytes)
            .unwrap_or(&UNITS[UNITS.len() - 1]);

        format!("{}{}", self.bytes / unit.bytes, unit.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(name: &str) -> Option<u64> {
        ByteSize::from_config_name(name).map(|size| size.bytes)
    }

    #[test]
    fn reads_binary_and_decimal_units() {
        assert_eq!(bytes("512MiB"), Some(512 * 1024 * 1024));
        assert_eq!(bytes("2GB"), Some(2_000_000_000));
        assert_eq!(bytes("100B"), Some(100));
    }

    #[test]
    fn rejects_a_missing_unit_or_number() {
        assert_eq!(bytes("512"), None);
        assert_eq!(bytes("MiB"), None);
        assert_eq!(bytes("1.5GiB"), None);
    }

    #[test]
    fn writes_the_largest_exact_unit() {
        assert_eq!(ByteSize { bytes: 1024 * 1024 }.config_name(), "1MiB");
        assert_eq!(ByteSize { bytes: 1_500_000 }.config_name(), "1500KB");
        assert_eq!(ByteSize { bytes: 0 }.config_name(), "0B");
    }
}
