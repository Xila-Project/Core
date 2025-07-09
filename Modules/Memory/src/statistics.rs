#[derive(Debug, Clone)]
pub struct Region_statistics_type {
    /// Total usable size of the heap region in bytes.
    pub size: usize,

    /// Currently used size of the heap region in bytes.
    pub used: usize,

    /// Free size of the heap region in bytes.
    pub free: usize,
}

#[derive(Debug)]
pub struct Statistics_type {
    /// Granular stats for all the configured memory regions.
    region_stats: [Option<Region_statistics_type>; 3],

    /// Total size of all combined heap regions in bytes.
    size: usize,

    /// Current usage of the heap across all configured regions in bytes.
    current_usage: usize,

    /// Estimation of the max used heap in bytes.
    #[cfg(feature = "Debug")]
    max_usage: usize,

    /// Estimation of the total allocated bytes since initialization.
    #[cfg(feature = "Debug")]
    total_allocated: usize,

    /// Estimation of the total freed bytes since initialization.
    #[cfg(feature = "Debug")]
    total_freed: usize,
}
