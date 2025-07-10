use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Port(u16);

impl Port {
    pub const ANY: Self = Self(0);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn into_inner(self) -> u16 {
        self.0
    }

    pub const fn from_inner(value: u16) -> Self {
        Self(value)
    }
}

impl Display for Port {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
