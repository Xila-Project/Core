use shared::flags;

flags! {
    pub enum CapabilityFlags: u8 {
        Executable,
        DirectMemoryAccess,
    }
}
