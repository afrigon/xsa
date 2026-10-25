use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TimeOfDay {
    // The star rises to the twilight depression below the horizon.
    Dawn,
    // The star's upper edge clears the horizon.
    Sunrise,
    // The star is at its highest.
    Noon,
    Sunset,
    Dusk,
    // The star is at its lowest.
    Midnight,
}

impl TimeOfDay {
    pub const ALL: [TimeOfDay; 6] = [
        TimeOfDay::Dawn,
        TimeOfDay::Sunrise,
        TimeOfDay::Noon,
        TimeOfDay::Sunset,
        TimeOfDay::Dusk,
        TimeOfDay::Midnight,
    ];

    pub fn name(self) -> &'static str {
        match self {
            TimeOfDay::Dawn => "dawn",
            TimeOfDay::Sunrise => "sunrise",
            TimeOfDay::Noon => "noon",
            TimeOfDay::Sunset => "sunset",
            TimeOfDay::Dusk => "dusk",
            TimeOfDay::Midnight => "midnight",
        }
    }
}

impl FromStr for TimeOfDay {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        TimeOfDay::ALL
            .into_iter()
            .find(|time_of_day| time_of_day.name() == text)
            .ok_or_else(|| {
                let names: Vec<&str> = TimeOfDay::ALL.iter().map(|time_of_day| time_of_day.name()).collect();
                format!("{text:?} is not a time of day: {}", names.join(", "))
            })
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
