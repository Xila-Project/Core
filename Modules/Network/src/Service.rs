use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Port_type(u16);

impl Port_type {
    pub const Any: Self = Self(0);

    pub const fn New(value: u16) -> Self {
        Self(value)
    }

    pub const fn Into_inner(self) -> u16 {
        self.0
    }

    pub const fn From_inner(value: u16) -> Self {
        Self(value)
    }
}

impl Display for Port_type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
