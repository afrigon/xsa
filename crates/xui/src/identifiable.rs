use std::hash::Hash;

// A value with a stable identity, like Swift's `Identifiable`: `ForEach` uses it to tell items apart.
pub trait Identifiable {
    type Id: Clone + Eq + Hash;

    fn id(&self) -> Self::Id;
}

impl<Item: Identifiable> Identifiable for &Item {
    type Id = Item::Id;

    fn id(&self) -> Item::Id {
        (**self).id()
    }
}

macro_rules! identifiable_integers {
    ($($integer:ty),+) => {
        $(
            impl Identifiable for $integer {
                type Id = $integer;

                fn id(&self) -> $integer {
                    *self
                }
            }
        )+
    };
}

identifiable_integers!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
