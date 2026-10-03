use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BodyCategory {
    Star,
    BrownDwarf,
    WhiteDwarf,
    NeutronStar,
    BlackHole,
    Planet,
    DwarfPlanet,
    Moon,
    Asteroid,
    Comet,
}

impl BodyCategory {
    pub const ALL: [BodyCategory; 10] = [
        BodyCategory::Star,
        BodyCategory::BrownDwarf,
        BodyCategory::WhiteDwarf,
        BodyCategory::NeutronStar,
        BodyCategory::BlackHole,
        BodyCategory::Planet,
        BodyCategory::DwarfPlanet,
        BodyCategory::Moon,
        BodyCategory::Asteroid,
        BodyCategory::Comet,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BodyCategory::Star => "star",
            BodyCategory::BrownDwarf => "brown-dwarf",
            BodyCategory::WhiteDwarf => "white-dwarf",
            BodyCategory::NeutronStar => "neutron-star",
            BodyCategory::BlackHole => "black-hole",
            BodyCategory::Planet => "planet",
            BodyCategory::DwarfPlanet => "dwarf-planet",
            BodyCategory::Moon => "moon",
            BodyCategory::Asteroid => "asteroid",
            BodyCategory::Comet => "comet",
        }
    }
}

impl FromStr for BodyCategory {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        BodyCategory::ALL
            .into_iter()
            .find(|category| category.name() == text)
            .ok_or_else(|| {
                let names: Vec<&str> = BodyCategory::ALL.iter().map(|category| category.name()).collect();
                format!("{text:?} is not a body category: {}", names.join(", "))
            })
    }
}

impl fmt::Display for BodyCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
