// The RAM the system can give processes without paging, as Windows reports it.
pub(crate) struct AvailableMemory;

// MEMORYSTATUSEX.
#[repr(C)]
#[derive(Default)]
struct MemoryStatus {
    length: u32,
    memory_load: u32,
    total_physical: u64,
    available_physical: u64,
    total_page_file: u64,
    available_page_file: u64,
    total_virtual: u64,
    available_virtual: u64,
    available_extended_virtual: u64,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalMemoryStatusEx(buffer: *mut MemoryStatus) -> i32;
}

impl AvailableMemory {
    pub fn new() -> anyhow::Result<AvailableMemory> {
        Ok(AvailableMemory)
    }

    pub fn bytes(&mut self) -> anyhow::Result<u64> {
        let mut status = MemoryStatus {
            length: size_of::<MemoryStatus>() as u32,
            ..MemoryStatus::default()
        };

        if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(status.available_physical)
    }
}
