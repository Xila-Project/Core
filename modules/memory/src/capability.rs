use core::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Capabilities(u8);

impl Capabilities {
    pub const EXECUTABLE_FLAG: u8 = 1 << 0;
    pub const DIRECT_MEMORY_ACCESS_FLAG: u8 = 1 << 1;

    pub const fn new(executable: bool, direct_memory_access: bool) -> Self {
        Capabilities(0)
            .set_executable(executable)
            .set_direct_memory_access(direct_memory_access)
    }

    pub const fn get_executable(&self) -> bool {
        self.0 & Capabilities::EXECUTABLE_FLAG != 0
    }

    pub const fn get_direct_memory_access(&self) -> bool {
        self.0 & Capabilities::DIRECT_MEMORY_ACCESS_FLAG != 0
    }

    pub const fn set_executable(mut self, value: bool) -> Self {
        if value {
            self.0 |= Capabilities::EXECUTABLE_FLAG;
        } else {
            self.0 &= !Capabilities::EXECUTABLE_FLAG;
        }
        self
    }

    pub const fn set_direct_memory_access(mut self, value: bool) -> Self {
        if value {
            self.0 |= Capabilities::DIRECT_MEMORY_ACCESS_FLAG;
        } else {
            self.0 &= !Capabilities::DIRECT_MEMORY_ACCESS_FLAG;
        }
        self
    }

    pub const fn is_subset_of(&self, other: Capabilities) -> bool {
        (self.0 & other.0) == self.0
    }

    pub const fn is_superset_of(&self, other: Capabilities) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn from_u8(value: u8) -> Self {
        Capabilities(value)
    }

    pub const fn to_u8(&self) -> u8 {
        self.0
    }
}

impl Display for Capabilities {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Capabilities: ")?;
        if self.get_executable() {
            write!(f, "Executable ")?;
        }
        if self.get_direct_memory_access() {
            write!(f, "Direct memory access ")?;
        }
        Ok(())
    }
}
