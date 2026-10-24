use bitcode::{Decode, Encode};

// How a time warp gets to its end.
#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq)]
pub enum WarpPace {
    // Time jumps there at once.
    Instant,
    // Time eases there over about this many real seconds, longer when the fastest time rate cannot cover it sooner.
    Eased { seconds: f64 },
}
