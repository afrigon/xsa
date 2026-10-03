#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BodyIndex {
    pub value: usize,
}

impl BodyIndex {
    pub fn next(self, count: usize) -> BodyIndex {
        BodyIndex {
            value: (self.value + 1) % count,
        }
    }

    pub fn previous(self, count: usize) -> BodyIndex {
        BodyIndex {
            value: (self.value + count - 1) % count,
        }
    }
}
