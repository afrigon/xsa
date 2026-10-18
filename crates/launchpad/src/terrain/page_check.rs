/// How deep `terrain inspect` checks a file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageCheck {
    /// The header and every page entry.
    Index,
    /// The index, and every page decompressed to its expected size.
    Pages,
}
