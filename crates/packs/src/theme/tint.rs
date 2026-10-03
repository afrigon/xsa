use anyhow::Context;

const TINT_STEP: u16 = 100;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tint {
    Value100,
    Value200,
    Value300,
    Value400,
    Value500,
    Value600,
    Value700,
    Value800,
    Value900,
    Value1000,
}

impl Tint {
    pub const ALL: [Tint; 10] = [
        Tint::Value100,
        Tint::Value200,
        Tint::Value300,
        Tint::Value400,
        Tint::Value500,
        Tint::Value600,
        Tint::Value700,
        Tint::Value800,
        Tint::Value900,
        Tint::Value1000,
    ];

    pub fn from_number(number: f64) -> anyhow::Result<Tint> {
        Tint::ALL
            .into_iter()
            .find(|tint| f64::from(tint.number()) == number)
            .with_context(|| format!("{number} is not a tint: use 100, 200, … 1000"))
    }

    pub fn number(self) -> u16 {
        (self.index() as u16 + 1) * TINT_STEP
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn inverse(self) -> Tint {
        Tint::ALL[Tint::ALL.len() - 1 - self.index()]
    }

    pub fn higher(self) -> Tint {
        Tint::ALL[(self.index() + 1).min(Tint::ALL.len() - 1)]
    }

    pub fn lower(self) -> Tint {
        Tint::ALL[self.index().saturating_sub(1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_round_trip() {
        for tint in Tint::ALL {
            assert_eq!(Tint::from_number(f64::from(tint.number())).unwrap(), tint);
        }
        assert!(Tint::from_number(50.0).is_err());
        assert!(Tint::from_number(550.0).is_err());
    }

    #[test]
    fn inverse_mirrors_the_scale() {
        assert_eq!(Tint::Value100.inverse(), Tint::Value1000);
        assert_eq!(Tint::Value500.inverse(), Tint::Value600);
    }

    #[test]
    fn higher_and_lower_stop_at_the_ends() {
        assert_eq!(Tint::Value1000.higher(), Tint::Value1000);
        assert_eq!(Tint::Value100.lower(), Tint::Value100);
        assert_eq!(Tint::Value500.higher(), Tint::Value600);
        assert_eq!(Tint::Value500.lower(), Tint::Value400);
    }
}
