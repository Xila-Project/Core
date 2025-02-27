/// Stats for a heap region
#[derive(Debug, Clone)]
pub struct Region_statistics_type {
    /// Total usable size of the heap region in bytes.
    pub Size: usize,

    /// Currently used size of the heap region in bytes.
    pub Used: usize,

    /// Free size of the heap region in bytes.
    pub Free: usize,
}
