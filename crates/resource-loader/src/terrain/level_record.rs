/// Where a face and level's page entries are: the level index holds one record per face and level.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LevelRecord {
    pub entries_offset: u64,
    pub entry_count: u64,
}

impl LevelRecord {
    /// The size of a record in bytes.
    pub const SIZE: usize = 16;
    const ENTRY_COUNT_OFFSET: usize = 8;

    pub fn read(bytes: &[u8; LevelRecord::SIZE]) -> LevelRecord {
        let (entries_offset, entry_count) = bytes.split_at(Self::ENTRY_COUNT_OFFSET);

        LevelRecord {
            entries_offset: u64::from_le_bytes(entries_offset.try_into().unwrap()),
            entry_count: u64::from_le_bytes(entry_count.try_into().unwrap()),
        }
    }
}
