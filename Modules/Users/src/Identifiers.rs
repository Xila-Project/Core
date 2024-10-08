use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct User_identifier_type(u16);

impl User_identifier_type {
    pub const Root: Self = Self::New(0);

    pub const Minimum: Self = Self::New(1);
    pub const Maximum: Self = Self::New(u16::MAX);

    pub const fn New(Identifier: u16) -> Self {
        Self(Identifier)
    }

    pub const fn Into_inner(self) -> u16 {
        self.0
    }
}

impl AddAssign<u16> for User_identifier_type {
    fn add_assign(&mut self, Other: u16) {
        self.0 += Other;
    }
}

impl Add<u16> for User_identifier_type {
    type Output = Self;

    fn add(self, Other: u16) -> Self {
        Self::New(self.0 + Other)
    }
}

impl From<u16> for User_identifier_type {
    fn from(Value: u16) -> Self {
        User_identifier_type::New(Value)
    }
}
impl From<User_identifier_type> for u16 {
    fn from(Value: User_identifier_type) -> Self {
        Value.Into_inner()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Group_identifier_type(u16);

impl Group_identifier_type {
    pub const Root: Self = Self::New(0);

    pub const Minimum: Self = Self::New(1);
    pub const Maximum: Self = Self::New(u16::MAX);

    pub const fn New(Identifier: u16) -> Self {
        Self(Identifier)
    }

    pub const fn Into_inner(self) -> u16 {
        self.0
    }
}

impl From<u16> for Group_identifier_type {
    fn from(Value: u16) -> Self {
        Group_identifier_type::New(Value)
    }
}
impl From<Group_identifier_type> for u16 {
    fn from(Value: Group_identifier_type) -> Self {
        Value.Into_inner()
    }
}

impl AddAssign<u16> for Group_identifier_type {
    fn add_assign(&mut self, Other: u16) {
        self.0 += Other;
    }
}

impl Add<u16> for Group_identifier_type {
    type Output = Self;

    fn add(self, Other: u16) -> Self {
        Self::New(self.0 + Other)
    }
}
