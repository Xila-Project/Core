use core::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Capabilities_type(u8);

impl Capabilities_type {
    const Executable_flag: u8 = 1 << 0;
    const Direct_memory_access_flag: u8 = 1 << 1;

    pub const fn New(Executable: bool, Direct_memory_access: bool) -> Self {
        Capabilities_type(0)
            .Set_executable(Executable)
            .Set_direct_memory_access(Direct_memory_access)
    }

    pub const fn Get_executable(&self) -> bool {
        self.0 & Capabilities_type::Executable_flag != 0
    }

    pub const fn Get_direct_memory_access(&self) -> bool {
        self.0 & Capabilities_type::Direct_memory_access_flag != 0
    }

    pub const fn Set_executable(mut self, value: bool) -> Self {
        if value {
            self.0 |= Capabilities_type::Executable_flag;
        } else {
            self.0 &= !Capabilities_type::Executable_flag;
        }
        self
    }

    pub const fn Set_direct_memory_access(mut self, value: bool) -> Self {
        if value {
            self.0 |= Capabilities_type::Direct_memory_access_flag;
        } else {
            self.0 &= !Capabilities_type::Direct_memory_access_flag;
        }
        self
    }

    pub const fn Is_subset_of(&self, Other: Capabilities_type) -> bool {
        (self.0 & Other.0) == self.0
    }

    pub const fn Is_superset_of(&self, Other: Capabilities_type) -> bool {
        (self.0 & Other.0) == Other.0
    }
}

impl Display for Capabilities_type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Capabilities: ")?;
        if self.Get_executable() {
            write!(f, "Executable ")?;
        }
        if self.Get_direct_memory_access() {
            write!(f, "Direct memory access ")?;
        }
        Ok(())
    }
}
