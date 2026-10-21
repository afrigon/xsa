// How much a terrain draw shades: from its terrain layers alone, or with surfaces where its pixels magnify the colour
// layer past its texels.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(in crate::renderer) enum ShadingRange {
    Far,
    Near,
}

impl ShadingRange {
    pub const ALL: [ShadingRange; 2] = [ShadingRange::Far, ShadingRange::Near];

    // Matches `ShadingRange` in terrain/surface/range.slang.
    pub fn shader_id(self) -> u32 {
        match self {
            ShadingRange::Far => 0,
            ShadingRange::Near => 1,
        }
    }
}
