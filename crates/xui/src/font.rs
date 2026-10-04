const REGULAR_WEIGHT: f32 = 400.0;

#[derive(Clone, PartialEq, Debug)]
pub struct Font {
    pub family: String,
    pub size: f32,
    pub weight: f32,
    pub letter_spacing: f32,
    pub features: Vec<String>,
}

impl Font {
    pub fn new(family: impl Into<String>, size: f32) -> Font {
        Font {
            family: family.into(),
            size,
            weight: REGULAR_WEIGHT,
            letter_spacing: 0.0,
            features: Vec::new(),
        }
    }

    pub fn weight(self, weight: f32) -> Font {
        Font { weight, ..self }
    }

    pub fn letter_spacing(self, letter_spacing: f32) -> Font {
        Font { letter_spacing, ..self }
    }

    pub fn features(self, features: Vec<String>) -> Font {
        Font { features, ..self }
    }
}
