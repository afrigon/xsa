use std::ops::{Add, AddAssign};

/// Bytes a resource occupies in video memory and in RAM.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryUsage {
    pub video: u64,
    pub ram: u64,
}

impl Add for MemoryUsage {
    type Output = MemoryUsage;

    fn add(self, other: MemoryUsage) -> MemoryUsage {
        MemoryUsage {
            video: self.video + other.video,
            ram: self.ram + other.ram,
        }
    }
}

impl AddAssign for MemoryUsage {
    fn add_assign(&mut self, other: MemoryUsage) {
        *self = *self + other;
    }
}
