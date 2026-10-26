use std::time::Duration;

/// What a holder of a mip-streamed resource needs: the finest level it shows, and how long the finer levels it no
/// longer needs stay resident, so returning to them soon costs nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MipDemand {
    /// The finest mip level wanted, 0 being the largest. A level beyond the chain wants only the smallest.
    pub finest_level: u32,
    pub retention: Duration,
}

impl MipDemand {
    /// What two holders need together: the finer level, kept the longer.
    pub fn combined(self, other: MipDemand) -> MipDemand {
        MipDemand {
            finest_level: self.finest_level.min(other.finest_level),
            retention: self.retention.max(other.retention),
        }
    }
}
