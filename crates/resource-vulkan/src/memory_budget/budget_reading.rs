use memory_vulkan::HeapUsage;

// What the budget decides from, read once per update.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BudgetReading {
    // The device-local heaps, as the driver reports them.
    pub video: HeapUsage,
    // Video memory already evicted that the driver still counts, until the deletion queue frees it.
    pub evicting_video: u64,
    // RAM every registered resource holds.
    pub held_ram: u64,
    // None when the system could not be read: RAM is then kept within its limit only.
    pub available_ram: Option<u64>,
}
