use miniserde::{Deserialize, Serialize};
use Users::{
    Group_identifier_inner_type, Group_identifier_type, User_identifier_inner_type,
    User_identifier_type,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group_type {
    Identifier: Group_identifier_inner_type,
    Name: String,
    Users: Vec<User_identifier_inner_type>,
}

impl Group_type {
    pub fn New(
        Identifier: Group_identifier_inner_type,
        Name: String,
        Users: Vec<User_identifier_inner_type>,
    ) -> Self {
        Self {
            Identifier,
            Name,
            Users,
        }
    }

    pub fn Get_identifier(&self) -> Group_identifier_type {
        Group_identifier_type::New(self.Identifier)
    }

    pub fn Get_name(&self) -> &str {
        &self.Name
    }

    pub fn Get_users(&self) -> &[User_identifier_type] {
        // Avoid to copy the vector since User_identifier_type is transparent to User_identifier_inner_type.
        unsafe { core::mem::transmute(self.Users.as_slice()) }
    }
}
