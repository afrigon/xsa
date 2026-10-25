use std::fmt;

use animation::Easing;
use usage::spec::ValueEnum;

// How an injected motion is spread over its duration.
#[derive(usage::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingName {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingName {
    pub fn easing(self) -> Easing {
        match self {
            EasingName::Linear => Easing::Linear,
            EasingName::EaseIn => Easing::EASE_IN,
            EasingName::EaseOut => Easing::EASE_OUT,
            EasingName::EaseInOut => Easing::EASE_IN_OUT,
        }
    }
}

// The derive lists each variant's word in declaration order.
impl fmt::Display for EasingName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(EasingName::CHOICES[*self as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_the_accepted_words() {
        for easing in [
            EasingName::Linear,
            EasingName::EaseIn,
            EasingName::EaseOut,
            EasingName::EaseInOut,
        ] {
            assert_eq!(EasingName::from_choice(&easing.to_string()), Some(easing));
        }
    }
}
