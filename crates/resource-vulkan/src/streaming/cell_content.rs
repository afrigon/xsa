// What a page holding a cell does with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellContent {
    Uploading,
    Resident,
    // Resident, and copying into another cell before its chunk is released.
    Moving,
}
