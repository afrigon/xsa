pub const METERS_PER_KILOMETER: f64 = 1_000.0;

pub struct LengthUnit {
    pub suffix: &'static str,
    pub meters: f64,
}

// Longest suffixes first: every unit ends with "m".
pub const LENGTH_UNITS: [LengthUnit; 4] = [
    LengthUnit {
        suffix: "Gm",
        meters: 1e9,
    },
    LengthUnit {
        suffix: "Mm",
        meters: 1e6,
    },
    LengthUnit {
        suffix: "km",
        meters: METERS_PER_KILOMETER,
    },
    LengthUnit {
        suffix: "m",
        meters: 1.0,
    },
];
