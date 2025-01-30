use std::ops::{Add, AddAssign};

pub type User_identifier_inner_type = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct User_identifier_type(User_identifier_inner_type);

impl User_identifier_type {
    pub const Root: Self = Self::New(0);

    pub const Minimum: Self = Self::New(1);
    pub const Maximum: Self = Self::New(User_identifier_inner_type::MAX);

    pub const fn New(Identifier: User_identifier_inner_type) -> Self {
        Self(Identifier)
    }

    pub const fn Into_inner(self) -> User_identifier_inner_type {
        self.0
    }
}

impl AddAssign<User_identifier_inner_type> for User_identifier_type {
    fn add_assign(&mut self, Other: User_identifier_inner_type) {
        self.0 += Other;
    }
}

impl Add<User_identifier_inner_type> for User_identifier_type {
    type Output = Self;

    fn add(self, Other: User_identifier_inner_type) -> Self {
        Self::New(self.0 + Other)
    }
}

impl From<User_identifier_inner_type> for User_identifier_type {
    fn from(Value: User_identifier_inner_type) -> Self {
        User_identifier_type::New(Value)
    }
}
impl From<User_identifier_type> for User_identifier_inner_type {
    fn from(Value: User_identifier_type) -> Self {
        Value.Into_inner()
    }
}

pub type Group_identifier_inner_type = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Group_identifier_type(Group_identifier_inner_type);

impl Group_identifier_type {
    pub const Root: Self = Self::New(0);

    pub const Minimum: Self = Self::New(1);
    pub const Maximum: Self = Self::New(Group_identifier_inner_type::MAX);

    pub const fn New(Identifier: Group_identifier_inner_type) -> Self {
        Self(Identifier)
    }

    pub const fn Into_inner(self) -> Group_identifier_inner_type {
        self.0
    }
}

impl From<Group_identifier_inner_type> for Group_identifier_type {
    fn from(Value: Group_identifier_inner_type) -> Self {
        Group_identifier_type::New(Value)
    }
}
impl From<Group_identifier_type> for Group_identifier_inner_type {
    fn from(Value: Group_identifier_type) -> Self {
        Value.Into_inner()
    }
}

impl AddAssign<Group_identifier_inner_type> for Group_identifier_type {
    fn add_assign(&mut self, Other: Group_identifier_inner_type) {
        self.0 += Other;
    }
}

impl Add<Group_identifier_inner_type> for Group_identifier_type {
    type Output = Self;

    fn add(self, Other: Group_identifier_inner_type) -> Self {
        Self::New(self.0 + Other)
    }
}
