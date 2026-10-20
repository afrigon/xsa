const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const SPLIT_MIX_INCREMENT: u64 = 0x9e37_79b9_7f4a_7c15;
const SPLIT_MIX_FIRST_MULTIPLIER: u64 = 0xbf58_476d_1ce4_e5b9;
const SPLIT_MIX_SECOND_MULTIPLIER: u64 = 0x94d0_49bb_1331_11eb;

// Where randomness starts. Every use derives its own seed from its place, so the client and the server agree by
// construction and no two uses share a pattern. The hashes are fixed algorithms, never the standard library's, which
// may change between Rust versions.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Seed {
    pub value: u64,
}

impl Seed {
    pub fn from_id(id: &str) -> Seed {
        Seed {
            value: Seed::split_mix(Seed::fnv(id)),
        }
    }

    pub fn derive(self, place: &str) -> Seed {
        Seed {
            value: Seed::split_mix(self.value ^ Seed::fnv(place)),
        }
    }

    fn fnv(text: &str) -> u64 {
        text.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
        })
    }

    fn split_mix(value: u64) -> u64 {
        let mut mixed = value.wrapping_add(SPLIT_MIX_INCREMENT);
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(SPLIT_MIX_FIRST_MULTIPLIER);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(SPLIT_MIX_SECOND_MULTIPLIER);
        mixed ^ (mixed >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_id_gives_the_same_seed() {
        assert_eq!(Seed::from_id("luna"), Seed::from_id("luna"));
        assert_ne!(Seed::from_id("luna"), Seed::from_id("earth"));
    }

    #[test]
    fn places_derive_distinct_seeds() {
        let body = Seed::from_id("luna");
        assert_ne!(body.derive("flat"), body.derive("slope"));
        assert_ne!(body.derive("flat"), body);
    }

    // Pins the algorithm: a changed hash would move every seeded pattern in saved games and between versions.
    #[test]
    fn the_hash_is_stable() {
        assert_eq!(Seed::fnv(""), FNV_OFFSET_BASIS);
        assert_eq!(Seed::fnv("a"), 0xaf63_dc4c_8601_ec8c);
    }
}
