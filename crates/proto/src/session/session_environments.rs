use super::SessionEnvironment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionEnvironments {
    bits: u8,
}

impl SessionEnvironments {
    pub const SINGLEPLAYER: SessionEnvironments = SessionEnvironments::only(SessionEnvironment::Singleplayer);
    pub const LAN_HOST: SessionEnvironments = SessionEnvironments::only(SessionEnvironment::LanHost);
    pub const DEDICATED: SessionEnvironments = SessionEnvironments::only(SessionEnvironment::Dedicated);
    pub const INTEGRATED: SessionEnvironments = Self::SINGLEPLAYER.union(Self::LAN_HOST);
    pub const MULTIPLAYER: SessionEnvironments = Self::LAN_HOST.union(Self::DEDICATED);
    pub const ANY: SessionEnvironments = Self::INTEGRATED.union(Self::DEDICATED);

    pub const fn only(environment: SessionEnvironment) -> SessionEnvironments {
        SessionEnvironments {
            bits: 1 << environment as u8,
        }
    }

    pub const fn union(self, other: SessionEnvironments) -> SessionEnvironments {
        SessionEnvironments {
            bits: self.bits | other.bits,
        }
    }

    pub fn contains(self, environment: SessionEnvironment) -> bool {
        self.bits & SessionEnvironments::only(environment).bits != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_contain_their_environments() {
        assert!(SessionEnvironments::INTEGRATED.contains(SessionEnvironment::Singleplayer));
        assert!(SessionEnvironments::INTEGRATED.contains(SessionEnvironment::LanHost));
        assert!(!SessionEnvironments::INTEGRATED.contains(SessionEnvironment::Dedicated));
        assert!(!SessionEnvironments::MULTIPLAYER.contains(SessionEnvironment::Singleplayer));
        assert!(SessionEnvironments::ANY.contains(SessionEnvironment::Dedicated));
        assert!(!SessionEnvironments::SINGLEPLAYER.contains(SessionEnvironment::LanHost));
    }
}
